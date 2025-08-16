/*
    ArduinoX86 Copyright 2022-2025 Daniel Balsom
    https://github.com/dbalsom/arduinoX86

    Permission is hereby granted, free of charge, to any person obtaining a
    copy of this software and associated documentation files (the “Software”),
    to deal in the Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, sublicense,
    and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.
*/

use super::{Config, InstructionSize, TestContext};
use crate::{display::print_regs_v2, gen_regs::TestRegisters, registers::Registers};
use anyhow::{anyhow, Context, Error};
use moo::types::{
    MooComparison,
    MooException,
    MooFileMetadata,
    MooRegisters,
    MooRegisters32,
    MooRegisters32Printer,
    MooRegistersInit,
    MooTestGenMetadata,
};
use std::{ffi::OsString, io::BufWriter, time::Instant};

use arduinox86_client::{
    CpuWidth,
    MemoryStrategy,
    ProgramState,
    RemoteCpuRegistersV2,
    RemoteCpuRegistersV3B,
    ServerCpuType,
    ServerFlags,
};

use moo::prelude::*;

use crate::{
    bus_ops::BusOps,
    cpu_common::{BusOp, BusOpType},
    cycles::MyServerCycleState,
    instruction::TestInstruction,
    state::{final_state_from_ops, initial_state_from_ops},
};
use anyhow::bail;
use iced_x86::{Mnemonic, OpKind};
use moo::types::{MooCpuType, MooRamEntry, MooRegisters16, MooRegisters16Printer, MooStateType};
use rand::{Rng, SeedableRng};

// Trace print macro that writes to bufwriter
#[macro_export]
macro_rules! trace_log {
    // take a mutable Context (or &mut Context) and a format+args
    ($ctx:expr, $($arg:tt)*) => {{
        // bring Write into scope so write!/writeln! work
        use std::io::Write;
        // write the formatted text plus a newline
        writeln!($ctx.trace_log, $($arg)*)
            .expect("failed to write to trace_log!");
    }};
}

#[macro_export]
macro_rules! trace_error {
    ($ctx:expr, $($arg:tt)*) => {{
        use std::io::Write;
        // 1) prefix
        write!($ctx.trace_log, "## ERROR: ")
            .expect("failed to write error prefix to trace_log");
        // 2) the user’s format + newline
        writeln!($ctx.trace_log, $($arg)*)
            .expect("failed to write to trace_log");
        // 3) also log via log::error!
        log::error!($($arg)*);
    }};
}

pub fn validate_tests(context: &mut TestContext, config: &Config) -> anyhow::Result<()> {
    let mut opcode_range_start: u8 = 0;
    let mut opcode_range_end: u8 = 0xFF;

    if config.test_gen.opcode_range.len() > 1 {
        opcode_range_start = config.test_gen.opcode_range[0];
        opcode_range_end = config.test_gen.opcode_range[1];

        println!(
            "Generating tests for opcodes from [{:02x} to {:02x}]",
            opcode_range_start, opcode_range_end
        );
    }
    else {
        log::error!("Invalid opcode range specified.");
        bail!("Invalid opcode range specified.");
    }

    // Tell ArduinoX86 to execute instructions automatically.
    context.client.set_flags(ServerFlags::EXECUTE_AUTOMATIC)?;
    // Set default serial debug state.
    context.client.enable_debug(config.test_exec.serial_debug_default)?;

    let mut last_opcode = opcode_range_start;
    for opcode in opcode_range_start..=opcode_range_end {
        let mut op_ext_start = 0;
        let mut op_ext_end = 0;
        let mut have_group_ext = false;
        if config.test_gen.group_opcodes.contains(&opcode) {
            have_group_ext = true;
            (op_ext_start, op_ext_end) = get_group_extension_range(config, opcode);
        }

        if config.test_gen.excluded_opcodes.contains(&opcode) {
            log::debug!("Skipping excluded opcode: {:02X}", opcode);
            continue;
        }
        if config.test_gen.prefixes.contains(&opcode) {
            log::debug!("Skipping prefix: {:02X}", opcode);
            continue;
        }

        for opcode_ext in op_ext_start..=op_ext_end {
            last_opcode = opcode;

            let mut op_ext_str = "".to_string();
            if have_group_ext {
                // If this is a group opcode, append the extension.
                op_ext_str = format!(".{:1X}", opcode_ext);
            }

            // Create the file path.
            let mut file_path = config.test_gen.test_output_dir.clone();
            let filename = OsString::from(format!("{:02X}{}.MOO", opcode, op_ext_str));
            file_path.push(filename.clone());

            // Create the trace file.
            let trace_filename = OsString::from(format!(
                "{:02X}{}{}",
                opcode,
                op_ext_str,
                config.test_gen.trace_file_suffix.display()
            ));
            let trace_file_path = config.test_gen.verify_trace_output_dir.join(trace_filename);

            // Open the trace file if it exists (and we are appending), otherwise create a new one.
            let trace_file = if !config.test_gen.append_file || !trace_file_path.exists() {
                log::debug!("Creating trace file {}", trace_file_path.to_string_lossy());
                std::fs::File::create(&trace_file_path)
                    .with_context(|| format!("Creating trace file: {}", trace_file_path.display()))?
            }
            else {
                log::debug!("Using existing trace file: {}", trace_file_path.to_string_lossy());
                std::fs::OpenOptions::new()
                    .append(true)
                    .open(&trace_file_path)
                    .with_context(|| format!("Opening existing trace file: {}", trace_file_path.display()))?
            };
            context.trace_log = BufWriter::new(trace_file);

            let mut test_file = MooTestFile::new(
                config.test_gen.moo_version,
                "C286".to_string(),
                config.test_gen.test_count,
            );

            // Open `file_path` for reading as a BufReader.
            match std::fs::File::open(&file_path) {
                Ok(file) => {
                    log::debug!("Appending to existing test file: {}", file_path.to_string_lossy());
                    let mut file_reader = std::io::BufReader::new(file);
                    test_file = MooTestFile::read(&mut file_reader)?;

                    if test_file.metadata().is_none() {
                        return Err(anyhow::anyhow!(
                            "Test file {} has no metadata.",
                            file_path.to_string_lossy()
                        ));
                    }

                    println!(
                        "Read {} tests from existing file: {}",
                        test_file.test_ct(),
                        file_path.to_string_lossy()
                    );
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        // If the file does not exist, we will create it later.
                        log::debug!("File {} not found, skipping...", file_path.to_string_lossy());
                        continue;
                    }
                    else {
                        return Err(anyhow::anyhow!("Error opening test file: {}", e));
                    }
                }
            }

            for test_num in 0..test_file.test_ct() {
                // Create unique instruction and initial register set for each test.
                // These should not change regardless of test attempt count.

                let mut gen_num: usize = 0;

                let file_seed = test_file.metadata().unwrap().file_seed;

                let tests = test_file.tests();
                let instruction_bytes = tests[test_num].bytes();

                let mut test_registers = TestRegisters::from(tests[test_num].initial_regs());
                let mut test_instruction = TestInstruction::from((InstructionSize::Sixteen, instruction_bytes));

                // Write initial memory state to device.
                let initial_mem = tests[test_num].initial_mem_state();

                write_initial_mem(context, &initial_mem.entries)?;

                // Set flow control end condition
                if config.test_gen.flow_control_opcodes.contains(&opcode) {
                    let flags = context.client.get_flags()?;
                    if flags & ServerFlags::HALT_AFTER_JUMP == 0 {
                        // Enable halt after jump if not already set.
                        context.client.set_flags(flags | ServerFlags::HALT_AFTER_JUMP)?;
                        log::debug!("Enabled HALT_AFTER_JUMP for opcode {:02X}", opcode);
                    }
                }

                let mut test_attempt_ct = 0;
                let mut test_result = generate_test(
                    context,
                    config,
                    test_num,
                    gen_num,
                    opcode,
                    have_group_ext.then_some(opcode_ext),
                    &test_instruction,
                    &mut test_registers,
                );

                while test_result.is_err() {
                    test_attempt_ct += 1;
                    trace_error!(
                        context,
                        "Failed to generate test for opcode {:02X}, attempt {}/{}: {}",
                        opcode,
                        test_attempt_ct,
                        config.test_exec.test_retry,
                        test_result.as_ref().err().unwrap()
                    );

                    if test_attempt_ct >= config.test_exec.test_retry {
                        let err_str = format!(
                            "Failed to generate test for opcode {:02X} after {} attempts: {}",
                            opcode,
                            test_attempt_ct,
                            test_result.as_ref().err().unwrap()
                        );
                        trace_error!(context, "{}", err_str);

                        gen_num += 1;
                        if gen_num < config.test_exec.max_gen as usize {
                            trace_log!(context, "Retrying with new instruction generation: {}", gen_num);
                        }
                        else {
                            trace_error!(context, "Max generation attempts reached for test number {}", test_num);
                        }

                        // Generate a new random instruction.
                        test_instruction = TestInstruction::new(
                            context,
                            &config.test_gen,
                            InstructionSize::Sixteen,
                            opcode,
                            have_group_ext.then_some(opcode_ext),
                            &test_registers,
                            test_num,
                            gen_num,
                        )?;
                        test_registers = TestRegisters::new(context, &config, opcode, test_num, gen_num);
                    }

                    test_result = generate_test(
                        context,
                        config,
                        test_num,
                        gen_num,
                        opcode,
                        have_group_ext.then_some(opcode_ext),
                        &test_instruction,
                        &mut test_registers,
                    );
                }

                // Validate the test result matches the saved test.

                if let Ok(test) = test_result {
                    // Check if the test matches the saved test.
                    if test.final_regs() != tests[test_num].final_regs() {
                        trace_error!(
                            context,
                            "Register mismatch for opcode {:02X} at test number {}!",
                            opcode,
                            test_num,
                        );
                        compare_registers(&test.final_regs(), tests[test_num].final_regs());
                        return Err(anyhow::anyhow!(
                            "Register mismatch for opcode {:02X} at test number {}",
                            opcode,
                            test_num
                        ));
                    }
                    else {
                        trace_log!(context, "{:02X}:{:05X} registers validated.", opcode, test_num);
                    }
                }
                else {
                    trace_error!(
                        context,
                        "Failed to validate test for opcode {:02X} at test number {}",
                        opcode,
                        test_num,
                    );
                    return Err(test_result.err().unwrap());
                }
            }
        }
    }

    Ok(())
}

pub fn compare_registers(regs0: &MooRegisters, regs1: &MooRegisters) {
    match (regs0, regs1) {
        (MooRegisters::Sixteen(regs0_inner), MooRegisters::Sixteen(regs1_inner)) => {
            compare_registers16(regs0_inner, regs1_inner);
        }

        _ => {
            println!("Incompatible register types for comparison!");
        }
    }
}

pub fn compare_registers16(regs0: &MooRegisters16, regs1: &MooRegisters16) {
    if regs0.ax != regs1.ax {
        println!("AX mismatch: {:04X} != {:04X}", regs0.ax, regs1.ax);
    }
    if regs0.bx != regs1.bx {
        println!("BX mismatch: {:04X} != {:04X}", regs0.bx, regs1.bx);
    }
    if regs0.cx != regs1.cx {
        println!("CX mismatch: {:04X} != {:04X}", regs0.cx, regs1.cx);
    }
    if regs0.dx != regs1.dx {
        println!("DX mismatch: {:04X} != {:04X}", regs0.dx, regs1.dx);
    }
    if regs0.sp != regs1.sp {
        println!("SP mismatch: {:04X} != {:04X}", regs0.sp, regs1.sp);
    }
    if regs0.bp != regs1.bp {
        println!("BP mismatch: {:04X} != {:04X}", regs0.bp, regs1.bp);
    }
    if regs0.si != regs1.si {
        println!("SI mismatch: {:04X} != {:04X}", regs0.si, regs1.si);
    }
    if regs0.di != regs1.di {
        println!("DI mismatch: {:04X} != {:04X}", regs0.di, regs1.di);
    }
    if regs0.cs != regs1.cs {
        println!("CS mismatch: {:04X} != {:04X}", regs0.cs, regs1.cs);
    }
    if regs0.ds != regs1.ds {
        println!("DS mismatch: {:04X} != {:04X}", regs0.ds, regs1.ds);
    }
    if regs0.es != regs1.es {
        println!("ES mismatch: {:04X} != {:04X}", regs0.es, regs1.es);
    }
    if regs0.ss != regs1.ss {
        println!("SS mismatch: {:04X} != {:04X}", regs0.ss, regs1.ss);
    }
    if regs0.ip != regs1.ip {
        println!("IP mismatch: {:04X} != {:04X}", regs0.ip, regs1.ip);
    }
    if regs0.flags != regs1.flags {
        println!("FLAGS mismatch: {:04X} != {:04X}", regs0.flags, regs1.flags);
    }
}

pub fn write_initial_mem(context: &mut TestContext, initial_mem: &[MooRamEntry]) -> anyhow::Result<()> {
    let mut last_mem_address = 0;
    let mut mem_vec: Vec<(u32, Vec<u8>)> = Vec::new();
    let mut consecutive_start_address = 0;
    let mut consecutive_bytes = Vec::new();
    // Make concurrent vectors out of consecutive memory entries.
    for entry in initial_mem {
        if entry.address == last_mem_address + 1 {
            // Consecutive entry.
            consecutive_bytes.push(entry.value);
        }
        else {
            // Push the previous consecutive entries, if any.
            if !consecutive_bytes.is_empty() {
                mem_vec.push((consecutive_start_address, consecutive_bytes.clone()));
                consecutive_bytes.clear();
            }
            consecutive_start_address = entry.address;
            consecutive_bytes.push(entry.value);
        }
        last_mem_address = entry.address;
    }

    // Push the last consecutive entries, if any.
    if !consecutive_bytes.is_empty() {
        mem_vec.push((consecutive_start_address, consecutive_bytes));
    }

    for span in mem_vec {
        log::debug!(
            "Writing initial memory at address {:08X} with {} bytes: {:02X?}",
            span.0,
            span.1.len(),
            span.1
        );
        context
            .client
            .set_memory(span.0, &span.1)
            .with_context(|| format!("Writing initial memory at address {:08X}", span.0))?;
    }
    Ok(())
}

pub fn gen_tests(context: &mut TestContext, config: &Config) -> anyhow::Result<()> {
    let mut opcode_range_start: u8 = 0;
    let mut opcode_range_end: u8 = 0xFF;

    context.gen_ct = 0;
    context.gen_start = Instant::now();

    for count_override in &config.test_gen.count_overrides {
        log::debug!(
            "Opcode count override: {:X?} -> {}",
            count_override.opcode_range,
            count_override.count
        );
    }

    if config.test_gen.opcode_range.len() > 1 {
        opcode_range_start = config.test_gen.opcode_range[0];
        opcode_range_end = config.test_gen.opcode_range[1];

        println!(
            "Generating tests for opcodes from [{:02x} to {:02x}]",
            opcode_range_start, opcode_range_end
        );
    }
    else {
        log::error!("Invalid opcode range specified.");
        bail!("Invalid opcode range specified.");
    }

    // Tell ArduinoX86 to execute instructions automatically.
    let mut server_flags = ServerFlags::EXECUTE_AUTOMATIC;

    if let MooCpuType::Intel80386Ex = config.test_gen.cpu_type {
        server_flags |= ServerFlags::USE_SMM;
    }

    context.client.set_flags(server_flags)?;
    // Set default serial debug state.
    context.client.enable_debug(config.test_exec.serial_debug_default)?;

    let prefix_byte: Option<u8> = None;
    let mut last_opcode = opcode_range_start;

    for opcode in opcode_range_start..=opcode_range_end {
        for size in &config.test_gen.gen_widths {
            let mut op_ext_start = 0;
            let mut op_ext_end = 0;
            let mut have_group_ext = false;
            if config.test_gen.group_opcodes.contains(&opcode) {
                have_group_ext = true;
                (op_ext_start, op_ext_end) = get_group_extension_range(config, opcode);
            }

            for opcode_ext in op_ext_start..=op_ext_end {
                last_opcode = opcode;

                // Reset mnemonic hashmap.
                context.mnemonic_set.clear();

                if config.test_gen.excluded_opcodes.contains(&opcode) {
                    log::debug!("Skipping excluded opcode: {:02X}", opcode);
                    continue;
                }
                if config.test_gen.prefixes.contains(&opcode) {
                    log::debug!("Skipping prefix: {:02X}", opcode);
                    continue;
                }

                let mut op_ext_str = "".to_string();
                if have_group_ext {
                    // If this is a group opcode, append the extension.
                    op_ext_str = format!(".{:1X}", opcode_ext);
                }

                // Create the output file path.
                let mut file_path = config.test_gen.test_output_dir.clone();
                let filename = OsString::from(format!("{:02X}{}.MOO", opcode, op_ext_str));
                file_path.push(filename.clone());

                // Create the trace file.
                let trace_filename = OsString::from(format!(
                    "{:02X}{}{}",
                    opcode,
                    op_ext_str,
                    config.test_gen.trace_file_suffix.display()
                ));
                let trace_file_path = config.test_gen.trace_output_dir.join(trace_filename);
                let trace_file = std::fs::File::create(&trace_file_path)
                    .with_context(|| format!("Creating trace file: {}", trace_file_path.display()))?;
                context.trace_log = BufWriter::new(trace_file);

                // Create the file seed.
                let mut file_seed: u64 = opcode as u64;
                if let Some(prefix_byte) = prefix_byte {
                    file_seed = file_seed | ((prefix_byte as u64) << 8);
                }
                file_seed <<= 3;
                file_seed |= (opcode_ext & 0x07) as u64;
                file_seed ^= config.test_gen.base_seed;

                context.file_seed = file_seed;
                let mut test_start_num = 0;

                let mut test_file = MooTestFile::new(
                    config.test_gen.moo_version,
                    config.test_gen.moo_arch.clone(),
                    config.test_gen.test_count,
                );

                let mut test_metadata = MooFileMetadata::new(
                    config.test_gen.set_version_major,
                    config.test_gen.set_version_minor,
                    config.test_gen.cpu_type.into(),
                    opcode as u32,
                )
                .with_file_seed(context.file_seed);

                // Open the file if append == true
                if config.test_gen.append_file {
                    // Open `filename` for reading as a BufReader.
                    match std::fs::File::open(&file_path) {
                        Ok(file) => {
                            log::debug!("Appending to existing test file: {}", file_path.to_string_lossy());
                            let mut file_reader = std::io::BufReader::new(file);
                            test_file = MooTestFile::read(&mut file_reader)?;

                            println!(
                                "Read {} tests from existing file: {}",
                                test_file.test_ct(),
                                file_path.to_string_lossy()
                            );

                            test_start_num = test_file.test_ct();
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::NotFound {
                                // If the file does not exist, we will create it later.
                                log::debug!(
                                    "File {} not found, creating new test file.",
                                    file_path.to_string_lossy()
                                );
                            }
                            else {
                                return Err(anyhow::anyhow!("Error opening test file: {}", e));
                            }
                        }
                    }
                };

                if test_start_num >= config.test_gen.test_count {
                    println!("Test file {} is complete. Skipping...", file_path.to_string_lossy());
                    continue;
                }

                let test_count = get_test_count(config, opcode);
                for test_num in test_start_num..test_count {
                    // Create unique instruction and initial register set for each test.
                    // These should not change regardless of test attempt count.

                    let mut test_result = generate_consistent_test(
                        context,
                        config,
                        test_num,
                        *size,
                        opcode,
                        have_group_ext,
                        opcode_ext,
                        config.test_exec.validate_count as usize,
                    );

                    if !context.dry_run {
                        if test_result.is_err() {
                            let err_msg = format!(
                                "Failed to generate test for opcode {:02X} at test number {}: {}",
                                opcode,
                                test_num,
                                test_result.as_ref().err().unwrap()
                            );
                            trace_error!(context, "{}", err_msg);
                            return Err(anyhow::anyhow!(err_msg));
                        }

                        // Add the test to the test file.
                        let test = test_result?;
                        test_file.add_test(test);
                        context.gen_ct += 1;
                    }
                }
                // Test generation is complete.

                // Log time taken
                context.gen_stop = Instant::now();
                if config.test_exec.show_gen_time {
                    let gen_duration = context.gen_stop.duration_since(context.gen_start);
                    println!(
                        "Generated {} tests in {:.2?} seconds ({} tests per second)",
                        context.gen_ct,
                        gen_duration,
                        context.gen_ct as f64 / gen_duration.as_secs_f64()
                    );
                }

                // Adjust final metadata with count...
                test_metadata = test_metadata.with_test_count(context.gen_ct as u32);
                // ... and with the most frequently seen mnemonic (to handle some tests that have invalid forms icedx86 won't decode).
                if let Some((mnemonic, count)) = context.mnemonic_set.iter().max_by_key(|entry| entry.1) {
                    log::debug!("Most frequent mnemonic: {} ({} times)", mnemonic, count);
                    test_metadata = test_metadata.with_mnemonic(mnemonic.to_string());
                }

                test_file.set_metadata(test_metadata);

                // Open the file as a Writer.
                log::debug!("Writing test file: {}", file_path.to_string_lossy());

                let file = std::fs::File::create(&file_path)?;
                let mut writer = BufWriter::new(file);

                test_file.write(&mut writer)?;
            }
        }
    }

    println!("Test generation complete at terminating opcode: {:02X}", last_opcode);

    Ok(())
}

fn generate_consistent_test(
    context: &mut TestContext,
    config: &Config,
    test_num: usize,
    size: InstructionSize,
    opcode: u8,
    have_group_ext: bool,
    opcode_ext: u8,
    required_matches: usize,
) -> Result<MooTest, Error> {
    let mut gen_num = 0;

    // Set flow control end condition
    if config.test_gen.flow_control_opcodes.contains(&opcode) {
        let flags = context.client.get_flags()?;
        if flags & ServerFlags::HALT_AFTER_JUMP == 0 {
            // Enable halt after jump if not already set.
            context.client.set_flags(flags | ServerFlags::HALT_AFTER_JUMP)?;
            log::debug!("Enabled HALT_AFTER_JUMP for opcode {:02X}", opcode);
        }
    }

    // We'll attempt to generate a test up to 'max_gen' times before giving up.
    // If we can't generate a test after that point, something has gone very wrong, like the
    // ArduinoX86 has crashed, the opcode is invalid, or we hit a major bug.
    while gen_num < config.test_exec.max_gen as usize {
        // Generate a fresh Register & Instruction pair.
        let mut test_registers = TestRegisters::new(context, config, opcode, test_num, gen_num);
        let test_instruction = TestInstruction::new(
            context,
            &config.test_gen,
            size,
            opcode,
            have_group_ext.then_some(opcode_ext),
            &test_registers,
            test_num,
            gen_num,
        )?;

        let mut test_attempt_ct = 0;
        let mut prev_test: Option<MooTest> = None;
        let mut match_count = 0;

        while test_attempt_ct < config.test_exec.test_retry {
            if context.dry_run {
                return Err(anyhow!("Don't generate tests in dry run mode").into());
            }

            let test_result = generate_test(
                context,
                config,
                test_num,
                gen_num,
                opcode,
                have_group_ext.then_some(opcode_ext),
                &test_instruction,
                &mut test_registers,
            );

            match test_result {
                Ok(test) => {
                    if let Some(prev) = &prev_test {
                        let comparison = prev.compare(&test);
                        let mut matched = false;
                        match comparison {
                            MooComparison::Equal => {
                                matched = true;
                                match_count += 1;
                                if match_count >= required_matches - 1 {
                                    trace_log!(
                                        context,
                                        "generate_consistent_test(): Test validation count met. Returning test."
                                    );
                                    return Ok(test);
                                }
                            }
                            MooComparison::RegisterMismatch => {
                                trace_error!(
                                    context,
                                    "generate_consistent_test(): Register mismatch with previous test."
                                );
                                compare_registers(&test.final_regs(), prev.final_regs());
                            }
                            MooComparison::MemoryAddressMismatch(prev, current) => {
                                trace_error!(
                                    context,
                                    "generate_consistent_test(): Memory address mismatch. Current: {:?} Previous: {:?}",
                                    current,
                                    prev
                                );
                            }
                            MooComparison::MemoryValueMismatch(prev, current) => {
                                trace_error!(
                                    context,
                                    "generate_consistent_test(): Memory value mismatch. Current: {:?} Previous: {:?}",
                                    current,
                                    prev
                                );
                            }
                            MooComparison::CycleCountMismatch(prev, current) => {
                                trace_error!(
                                    context,
                                    "generate_consistent_test(): Cycle count mismatch. Current: {} Previous: {}",
                                    current,
                                    prev
                                );
                            }
                            MooComparison::CycleAddressMismatch(prev, current) => {
                                trace_error!(
                                    context,
                                    "generate_consistent_test(): Cycle address mismatch. Current: {:06X} Previous: {:06X}",
                                    current,
                                    prev
                                );
                            }
                            MooComparison::CycleBusMismatch(prev, current) => {
                                trace_error!(
                                    context,
                                    "generate_consistent_test(): Cycle bus mismatch. Current: {:1X} Previous: {:1X}",
                                    current,
                                    prev
                                );
                            }
                            MooComparison::ALEMismatch(cycle_n, prev, current) => {
                                trace_error!(
                                    context,
                                    "generate_consistent_test(): ALE mismatch at cycle {}. Current: {} Previous: {}",
                                    cycle_n,
                                    current,
                                    prev
                                );
                            }
                        }

                        if !matched {
                            trace_log!(
                                context,
                                "Test passed but did not match previous. Resetting match count."
                            );
                            match_count = 0;
                        }
                    }
                    else {
                        // First result
                        match_count = 0;
                    }
                    prev_test = Some(test);
                }

                Err(e) => {
                    trace_error!(
                        context,
                        "Failed to generate test for opcode {:02X}, attempt {}: {}",
                        opcode,
                        test_attempt_ct + 1,
                        e
                    );
                    match_count = 0;
                    prev_test = None;
                }
            }

            test_attempt_ct += 1;
        }

        gen_num += 1;
        trace_log!(
            context,
            "Retrying with new instruction generation (attempt {}/{})",
            gen_num,
            config.test_exec.max_gen
        );
    }

    let error_msg = format!(
        "Failed to generate consistent test for opcode {:02X} after {} instruction generations",
        opcode, config.test_exec.max_gen
    );
    trace_error!(context, "{}", error_msg);
    Err(anyhow::anyhow!(error_msg).into())
}

pub fn get_test_count(config: &Config, opcode: u8) -> usize {
    for ct_override in &config.test_gen.count_overrides {
        let [min, max] = &ct_override.opcode_range[..]
        else {
            continue;
        };
        if opcode >= *min && opcode <= *max {
            log::debug!(
                "Using test count override for opcode {:02X}: {}",
                opcode,
                ct_override.count
            );
            return std::cmp::min(config.test_gen.test_count, ct_override.count);
        }
    }
    log::debug!("Using default test count of {}", config.test_gen.test_count);
    config.test_gen.test_count
}

pub fn get_group_extension_range(config: &Config, opcode: u8) -> (u8, u8) {
    for ext_override in &config.test_gen.group_extension_overrides {
        if ext_override.opcode == opcode {
            return (
                ext_override.group_extension_range[0],
                ext_override.group_extension_range[1],
            );
        }
    }
    (
        config.test_gen.group_extension_range[0],
        config.test_gen.group_extension_range[1],
    )
}

pub fn log_instruction(
    context: &mut TestContext,
    config: &Config,
    test_num: usize,
    opcode: u8,
    op_ext: Option<u8>,
    test_instruction: &TestInstruction,
    test_registers: &TestRegisters,
) {
    let mut op_ext_str = String::new();
    if let Some(ext) = op_ext {
        // If this is a group opcode, append the extension.
        op_ext_str = format!(".{:1X}", ext);
    }

    let instruction_log_string = format!(
        "{:05} | {:04X}:{:04X} | {:02X}{} {:<35} │ {:02X?}",
        test_num,
        test_registers.regs.cs(),
        test_registers.regs.ip(),
        opcode,
        op_ext_str,
        test_instruction.name(),
        test_instruction.instr_bytes(),
    );

    if config.test_exec.print_instruction {
        println!("{}", instruction_log_string);
    }

    let bar = "----------------------------------------------------------------------------------------------------";
    trace_log!(context, ">>> {}", bar);
    trace_log!(context, ">>> Generating test {}", instruction_log_string);
    trace_log!(
        context,
        ">>> Op1:{:?} Op2:{:?}",
        test_instruction.op0_kind(),
        test_instruction.op1_kind()
    );
    trace_log!(context, ">>> {}", bar);

    // trace_log!(
    //     context,
    //     "Sequence bytes: {:02X?}",
    //     test_instruction.sequence_bytes()
    // );

    match &test_registers.regs {
        Registers::V2(regs) => {
            let moo_registers = MooRegisters16::try_from(regs).expect("Failed to convert registers to MooRegisters");

            trace_log!(
                context,
                "{}",
                MooRegisters16Printer {
                    regs: &moo_registers,
                    cpu_type: config.test_gen.cpu_type,
                    diff: None,
                }
            );
        }
        Registers::V3A(regs) => {
            let moo_registers = MooRegisters32::try_from(regs).expect("Failed to convert registers to MooRegisters");

            trace_log!(
                context,
                "{}",
                MooRegisters32Printer {
                    regs: &moo_registers,
                    cpu_type: config.test_gen.cpu_type,
                    diff: None,
                }
            );
        }
        _ => {
            unimplemented!(
                "Unsupported register set type for logging: {:?}",
                context.register_set_type
            );
        }
    }
}

pub fn generate_test(
    context: &mut TestContext,
    config: &Config,
    test_num: usize,
    gen_num: usize,
    opcode: u8,
    op_ext: Option<u8>,
    test_instruction: &TestInstruction,
    test_registers: &mut TestRegisters,
) -> anyhow::Result<MooTest> {
    log_instruction(
        context,
        config,
        test_num,
        opcode,
        op_ext,
        test_instruction,
        test_registers,
    );

    if context.dry_run {
        bail!("Dry run mode enabled, skipping test generation.");
    }

    // Mask CX register if the instruction has REP/REPNE prefix.
    // ---------------------------------------------------------------------------------------------
    if test_instruction.iced_instruction().has_rep_prefix() || test_instruction.iced_instruction().has_repne_prefix() {
        // If the instruction has a REP or REPNE prefix, log it.
        trace_log!(
            context,
            "Instruction {} has REP/REPNE prefix. Masking CX with {:04X}",
            test_instruction.name(),
            config.test_gen.rep_cx_mask
        );

        let cx = test_registers.regs.cx();
        test_registers.regs.set_cx(cx & config.test_gen.rep_cx_mask);
    }

    // Enable serial debug if configured.
    // ---------------------------------------------------------------------------------------------
    if Some(test_num) == config.test_exec.serial_debug_test {
        log::debug!("Enabling serial debug for test number {}", test_num);
        context.client.enable_debug(true)?;
    }
    else {
        context.client.enable_debug(config.test_exec.serial_debug_default)?;
    }

    // Generate test seed.
    // ---------------------------------------------------------------------------------------------
    let mut rng = rand::rngs::StdRng::seed_from_u64(context.file_seed);
    let mut test_seed: u64 = rng.random();
    for _ in 0..test_num {
        test_seed = rng.random();
    }

    let gen_metadata = MooTestGenMetadata {
        seed:   test_seed,
        gen_ct: gen_num as u16,
    };

    // Set memory seed.
    // ---------------------------------------------------------------------------------------------
    context.client.randomize_memory(test_seed as u32)?;

    // Determine the memory strategy based on the zero and ff chances.
    // ---------------------------------------------------------------------------------------------
    let strategy_chance: f32 = rng.random();
    let strategy = if strategy_chance < config.test_gen.mem_zero_chance {
        // Use zero memory strategy.
        trace_log!(context, "Using zero memory strategy");
        MemoryStrategy::Zero
    }
    else if strategy_chance < config.test_gen.mem_zero_chance + config.test_gen.mem_ones_chance {
        // Use ff memory strategy.
        trace_log!(context, "Using ff memory strategy");
        MemoryStrategy::Ones
    }
    else {
        // Use random memory strategy.
        trace_log!(context, "Using random memory strategy");
        MemoryStrategy::Random
    };

    // Set memory strategy on the client.
    context.client.set_memory_strategy(
        strategy,
        config.test_gen.mem_strategy_start,
        config.test_gen.mem_strategy_end,
    )?;

    // Upload the instruction sequence.
    log::trace!("Uploading instruction sequence...");
    context
        .client
        .set_memory(test_registers.instruction_address, test_instruction.sequence_bytes())?;

    // Fix up memory if necessary.
    adjust_memory(context, test_seed, test_instruction, test_registers)?;

    // Load the registers onto the Arduino.
    // ---------------------------------------------------------------------------------------------

    // Reset cursor before writing to buffer!
    context.load_register_buffer.set_position(0);
    test_registers.regs.to_buffer(&mut context.load_register_buffer);

    let mut load_attempt_ct = 1;
    log::trace!(
        "Uploading registers, attempt {}/{}",
        load_attempt_ct,
        config.test_exec.load_retry
    );

    if let Err(e) = context
        .client
        .load_registers_from_buf(context.register_set_type, context.load_register_buffer.get_ref())
    {
        // If the load fails, retry up to `config.test_exec.load_retry` times.
        while load_attempt_ct < config.test_exec.load_retry {
            load_attempt_ct += 1;
            log::trace!(
                "Retrying register upload, attempt {}/{}",
                load_attempt_ct,
                config.test_exec.load_retry
            );
            if context
                .client
                .load_registers_from_buf(context.register_set_type, context.load_register_buffer.get_ref())
                .is_ok()
            {
                break;
            }
        }
        if load_attempt_ct >= config.test_exec.load_retry {
            bail!("Failed to upload registers after {} attempts: {}", load_attempt_ct, e);
        }
    }

    // Poll program state until finished with execution.
    // ---------------------------------------------------------------------------------------------
    let mut state = context.client.get_program_state()?;
    let mut test_timeout = false;
    let start_time = Instant::now();
    while !matches!(
        state,
        ProgramState::StoreDone | ProgramState::Shutdown | ProgramState::Error
    ) {
        // Sleep for a little bit so we're not spamming the Arduino.
        std::thread::sleep(std::time::Duration::from_millis(config.test_exec.polling_sleep.into()));

        let millis = start_time.elapsed().as_millis() as u32;
        if millis > config.test_exec.test_timeout {
            log::error!(
                "Test timeout reached after {} ms, program state is: {:?}",
                millis,
                state
            );
            test_timeout = true;
            break;
        }
        state = context.client.get_program_state()?;
    }

    if matches!(state, ProgramState::Error) {
        log::error!("Error executing instruction: {}", context.client.get_last_error()?);

        context.last_program_state = Some(ProgramState::Error);
        return Err(anyhow::anyhow!(
            "Error executing instruction: {}",
            context.client.get_last_error()?
        ));
    }

    if matches!(state, ProgramState::Shutdown) {
        log::error!("Shutdown executing instruction: {}", context.client.get_last_error()?);

        context.last_program_state = Some(ProgramState::Shutdown);
        return Err(anyhow::anyhow!(
            "Shutdown executing instruction: {}",
            context.client.get_last_error()?
        ));
    }

    // Read the registers back from the Arduino.
    // ---------------------------------------------------------------------------------------------
    log::trace!("Reading registers back from ArduinoX86...");
    let reg_type = context
        .client
        .store_registers_to_buf(&mut context.store_register_buffer)
        .map_err(|e| anyhow::anyhow!("Error reading registers: {}", e))?;

    let final_regs = match reg_type {
        0x0 => {
            // V1 registers
            unimplemented!()
        }
        0x1 => {
            // V2 registers
            let regs_v2 = RemoteCpuRegistersV2::try_from(context.store_register_buffer.as_slice())
                .map_err(|e| anyhow::anyhow!("Error parsing V2 registers: {}", e))?;

            if config.test_exec.print_final_regs {
                print_regs_v2(&regs_v2, config.test_gen.cpu_type.into());
            }
            Registers::V2(regs_v2)
        }
        0x3 => {
            // V3B registers
            let regs_v3b = RemoteCpuRegistersV3B::try_from(context.store_register_buffer.as_slice())
                .map_err(|e| anyhow::anyhow!("Error parsing V3B registers: {}", e))?;

            if config.test_exec.print_final_regs {
                //print_regs_v3b(&regs_v3b, config.test_gen.cpu_type.into());
            }
            Registers::V3B(regs_v3b)
        }
        _ => {
            log::error!("Unknown register set type: {}", reg_type);
            bail!("Unknown register set type: {}", reg_type);
        }
    };

    // Read the cycle states from ArduinoX86.
    // ---------------------------------------------------------------------------------------------
    log::trace!("Reading cycle states from ArduinoX86...");
    let cycle_states = context.client.get_cycle_states()?;
    log::trace!("Got {} cycle states!", cycle_states.len(),);

    let mut my_cycle_vec = Vec::new();

    // Convert cycle states to MooCycleStates.
    let mut moo_cycle_states = Vec::with_capacity(cycle_states.len());
    for cycle_state in &cycle_states {
        let my_cycle = match config.test_gen.cpu_type {
            MooCpuType::Intel80286 => MyServerCycleState::State286(cycle_state.clone()),
            MooCpuType::Intel80386Ex => MyServerCycleState::State386Ex(cycle_state.clone()),
            _ => unimplemented!(
                "Unsupported CPU type for cycle state conversion: {:?}",
                config.test_gen.cpu_type
            ),
        };
        my_cycle_vec.push(my_cycle.clone());
        moo_cycle_states.push(MooCycleState::from(my_cycle));
    }

    log_cycle_states(context, &moo_cycle_states);

    // Collect BusOps from cycle states.
    // ---------------------------------------------------------------------------------------------
    let bus_ops = BusOps::from(my_cycle_vec.as_slice());
    log::trace!("Got {} bus operations from cycles", bus_ops.len(),);
    bus_ops.log(context);

    if let Err(e) = bus_ops.validate(
        config,
        &test_registers.regs,
        opcode,
        test_instruction.iced_instruction(),
        test_instruction.op0_kind(),
        test_instruction.op1_kind(),
    ) {
        log::error!("Bus operation validation failed: {}", e);
        trace_log!(context, "Bus operation validation failed: {}", e);
        return Err(e);
    }

    if let Err(e) = validate_regs(&final_regs) {
        log::error!("Register validation failed: {}", e);
        trace_log!(context, "Register validation failed: {}", e);
        return Err(e);
    }

    if let Err(e) = validate_register_delta(
        test_instruction.iced_instruction().mnemonic(),
        &test_registers.regs,
        &final_regs,
    ) {
        log::error!("Register delta validation failed: {}", e);
        trace_log!(context, "Register delta validation failed: {}", e);
        return Err(e);
    }

    // Calculate initial memory state from bus operations.
    // ---------------------------------------------------------------------------------------------
    let initial_state = initial_state_from_ops(
        CpuWidth::from(context.server_cpu),
        test_registers.regs.cs_base(),
        test_registers.regs.ip(),
        test_instruction.sequence_bytes(),
        0,
        &bus_ops,
    )?;

    log::trace!("Got {} initial RAM entries", initial_state.initial_ram.len());

    // Detect any exceptions from bus operations.
    // ---------------------------------------------------------------------------------------------
    let exception = bus_ops.detect_exception(context.server_cpu.into());

    if let Some(exception) = &exception {
        log::trace!("Detected exception: {}", exception.exception_num);

        trace_log!(context, "Detected exception: {}", exception.exception_num);
        trace_log!(context, "Flags on stack at {:06X}", exception.flag_address);
    }

    // Log final register state.
    // ---------------------------------------------------------------------------------------------
    match config.test_gen.cpu_type {
        MooCpuType::Intel80286 => {
            trace_log!(
                context,
                "{}",
                MooRegisters16Printer {
                    regs: &MooRegisters16::try_from(&final_regs)
                        .expect("Failed to convert final registers to MooRegisters16"),
                    cpu_type: config.test_gen.cpu_type,
                    diff: None,
                }
            );
        }
        MooCpuType::Intel80386Ex => {
            trace_log!(
                context,
                "{}",
                MooRegisters32Printer {
                    regs: &MooRegisters32::try_from(&final_regs)
                        .expect("Failed to convert final registers to MooRegisters32"),
                    cpu_type: config.test_gen.cpu_type,
                    diff: None,
                }
            );
        }
        _ => {}
    }

    // Calculate final memory state from initial state and bus operations.
    // ---------------------------------------------------------------------------------------------
    let final_ram = final_state_from_ops(initial_state.initial_state, &bus_ops)?;

    // Create the initial test state.
    let initial_state = create_state(&test_registers.regs, None, &initial_state.initial_ram);
    // Create the final test state.
    let final_state = create_state(&test_registers.regs, Some(&final_regs), &final_ram);

    // Add the mnemonic to the hash map.
    context
        .mnemonic_set
        .entry(test_instruction.mnemonic().into())
        .and_modify(|e| *e += 1)
        .or_insert(1);

    // Create the test case.
    let test = MooTest::new(
        test_instruction.name().into(),
        Some(gen_metadata),
        test_instruction.sequence_bytes(),
        initial_state,
        final_state,
        &moo_cycle_states,
        exception,
    );

    Ok(test)
}

pub fn adjust_memory(
    context: &mut TestContext,
    test_seed: u64,
    test_instruction: &TestInstruction,
    test_registers: &mut TestRegisters,
) -> anyhow::Result<()> {
    // If the instruction is POPF, we need to generate a flag value without the trap flag.
    match test_instruction.iced_instruction().mnemonic() {
        Mnemonic::Popf => {
            // Generate a random flag value without the trap flag.
            let mut rng = rand::rngs::StdRng::seed_from_u64(test_seed);
            let mut flags = rng.random::<u16>() & !0x0100; // Clear the trap flag (bit 8)

            // Calculate the stack address.
            let stack_address = test_registers.regs.stack_address();
            // Write the flags to the stack.
            context.client.set_memory(stack_address, &flags.to_le_bytes())?;
        }
        Mnemonic::Iret => {
            // Generate a random flag value without the trap flag.
            let mut rng = rand::rngs::StdRng::seed_from_u64(test_seed);
            let mut flags = rng.random::<u16>() & !0x0100; // Clear the trap flag (bit 8)

            // Calculate the stack address. It's +4 because we need to write the flags, CS, and IP.
            let mut stack_address = test_registers.regs.ss_base();
            stack_address += test_registers.regs.sp().wrapping_add(4) as u32;

            // Write the flags to the stack.
            context.client.set_memory(stack_address, &flags.to_le_bytes())?;
        }
        _ => {}
    }
    Ok(())
}

pub fn log_bus_ops(context: &mut TestContext, bus_ops: &[BusOp]) {
    trace_log!(context, "Bus operations ({})", bus_ops.len());
    for (i, bus_op) in bus_ops.iter().enumerate() {
        trace_log!(
            context,
            "{:02}: Addr: {:06X}, Data: {:04X?}, Type: {:?}",
            i,
            bus_op.addr,
            bus_op.data,
            bus_op.op_type
        );
    }
}

pub fn log_cycle_states(context: &mut TestContext, cycles: &[MooCycleState]) {
    for cycle in cycles {
        trace_log!(context, "{}", cycle);
    }
}

pub fn create_state(initial_regs: &Registers, final_regs: Option<&Registers>, ram: &Vec<[u32; 2]>) -> MooTestState {
    let initial_reg_init = MooRegistersInit::from(initial_regs);
    let final_reg_init = final_regs.map(MooRegistersInit::from);

    // let state_regs = if let Some(final_regs) = final_reg_init {
    //     // If we have final regs, compute the difference.
    //     MooRegisters1Init::from((&initial_reg_init, &final_regs))
    // } else {
    //     initial_reg_init
    // };

    let mut ram_vec: Vec<MooRamEntry> = Vec::with_capacity(ram.len());
    for entry in ram {
        ram_vec.push(MooRamEntry {
            address: entry[0],
            value:   entry[1] as u8,
        });
    }

    let state_type = if final_regs.is_some() {
        MooStateType::Final
    }
    else {
        MooStateType::Initial
    };

    let test_state = MooTestState::new(
        state_type,
        &initial_reg_init,
        final_reg_init.as_ref(),
        Vec::new(),
        ram_vec,
    );

    if !test_state.regs().is_valid() {
        log::error!("Invalid registers in test state!");
        panic!("Invalid registers in test state");
    }

    test_state
}

pub fn validate_regs(registers: &Registers) -> anyhow::Result<()> {
    let moo_registers = MooRegisters::try_from(registers)
        .map_err(|e| anyhow::anyhow!("Failed to convert registers to MooRegisters: {}", e))?;

    // Check for reserved bit. Flags shouldn't be 0.
    let flags = moo_registers.flags();
    if flags & 0x0002 == 0 {
        // Reserved bit is not set.
        return Err(anyhow::anyhow!("Reserved bit in flags is not set: {:04X}", flags,));
    }

    Ok(())
}

pub fn validate_register_delta(
    mnemonic: Mnemonic,
    initial_regs: &Registers,
    final_regs: &Registers,
) -> anyhow::Result<()> {
    let moo_initial = MooRegisters::try_from(initial_regs)
        .map_err(|e| anyhow::anyhow!("Failed to convert initial registers: {}", e))?;
    let moo_final =
        MooRegisters::try_from(final_regs).map_err(|e| anyhow::anyhow!("Failed to convert final registers: {}", e))?;

    let mut error = false;

    if let (MooRegisters::Sixteen(moo_initial_i), MooRegisters::Sixteen(moo_final_i)) = (moo_initial, moo_final) {
        if !matches!(mnemonic, Mnemonic::Xchg) {
            if (moo_initial_i.ax != moo_initial_i.cx) && (moo_final_i.ax == moo_initial_i.cx) {
                error = true;
            }
            if (moo_initial_i.cx != moo_initial_i.dx) && (moo_final_i.cx == moo_initial_i.dx) {
                error = true;
            }
            if (moo_initial_i.dx != moo_initial_i.bx) && (moo_final_i.dx == moo_initial_i.bx) {
                error = true;
            }
            if (moo_initial_i.bx != moo_initial_i.sp) && (moo_final_i.bx == moo_initial_i.sp) {
                error = true;
            }
            if (moo_initial_i.sp != moo_initial_i.bp) && (moo_final_i.sp == moo_initial_i.bp) {
                error = true;
            }
            if (moo_initial_i.bp != moo_initial_i.si) && (moo_final_i.bp == moo_initial_i.si) {
                error = true;
            }
            if (moo_initial_i.si != moo_initial_i.di) && (moo_final_i.si == moo_initial_i.di) {
                error = true;
            }
            if (moo_initial_i.di != moo_initial_i.es) && (moo_final_i.di == moo_initial_i.es) {
                error = true;
            }
            if (moo_initial_i.es != moo_initial_i.cs) && (moo_final_i.es == moo_initial_i.cs) {
                error = true;
            }
            if (moo_initial_i.cs != moo_initial_i.ss) && (moo_final_i.cs == moo_initial_i.ss) {
                error = true;
            }
            if (moo_initial_i.ss != moo_initial_i.ds) && (moo_final_i.ss == moo_initial_i.ds) {
                error = true;
            }
        }
    }

    if error {
        log::error!("Possible off-by-one STOREALL register error detected!");
        return Err(anyhow::anyhow!("Possible off-by-one STOREALL register error detected!"));
    }
    Ok(())
}
