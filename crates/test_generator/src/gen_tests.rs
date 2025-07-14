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

use super::{Config, TestContext};
use crate::display::print_regs_v2;
use crate::gen_regs::TestRegisters;
use crate::registers::Registers;
use anyhow::{anyhow, Context, Error};
use std::ffi::OsString;
use std::time::Instant;

use ard808x_client::{CpuWidth, MemoryStrategy, ProgramState, RemoteCpuRegistersV2, ServerFlags};

use moo::prelude::*;

use crate::cpu_common::{BusOp, BusOpType};
use crate::cycles::MyServerCycleState;
use crate::instruction::TestInstruction;
use crate::state::{final_state_from_ops, initial_state_from_ops};
use anyhow::bail;
use iced_x86::{Mnemonic, OpKind};
use moo::types::{MooCpuType, MooRamEntry, MooRegisters1, MooRegisters1Printer, MooStateType};
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

pub fn gen_tests(context: &mut TestContext, config: &Config) -> anyhow::Result<()> {
    let mut opcode_range_start: u8 = 0;
    let mut opcode_range_end: u8 = 0xFF;

    context.gen_start = Instant::now();

    for count_override in &config.test_gen.count_overrides {
        log::debug!(
            "Opcode range override: {:X?} -> {}",
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
    } else {
        log::error!("Invalid opcode range specified.");
        bail!("Invalid opcode range specified.");
    }

    // Tell ArduinoX86 to execute instructions automatically.
    context.client.set_flags(ServerFlags::EXECUTE_AUTOMATIC)?;
    // Set default serial debug state.
    context
        .client
        .enable_debug(config.test_exec.serial_debug_default)?;

    let prefix_byte: Option<u8> = None;
    let mut last_opcode = opcode_range_start;

    for opcode in opcode_range_start..=opcode_range_end {
        let mut op_ext_start = 0;
        let mut op_ext_end = 0;
        let mut have_group_ext = false;
        if config.test_gen.group_opcodes.contains(&opcode) {
            have_group_ext = true;
            (op_ext_start, op_ext_end) = get_group_extension_range(config, opcode);
        }

        for opcode_ext in op_ext_start..=op_ext_end {
            last_opcode = opcode;

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
            let mut file_path = config.test_gen.output_dir.clone();
            let filename = OsString::from(format!("{:02X}{}.MOO", opcode, op_ext_str));

            file_path.push(filename.clone());

            // Create the file seed.
            let mut file_seed: u64 = opcode as u64;
            if let Some(prefix_byte) = prefix_byte {
                file_seed = file_seed | ((prefix_byte as u64) << 8);
            }
            file_seed <<= 3;
            file_seed ^= config.test_gen.base_seed;

            // Create a PRNG based on the file seed.
            let mut rng = rand::rngs::StdRng::seed_from_u64(file_seed);

            let mut test_start_num = 0;

            let mut test_file = MooTestFile::new(
                config.test_gen.moo_version,
                "C286".to_string(),
                config.test_gen.test_count,
            );

            // Open the file if append == true
            if config.test_gen.append_file {
                // Open `filename` for reading as a BufReader.
                match std::fs::File::open(&file_path) {
                    Ok(file) => {
                        log::debug!(
                            "Appending to existing test file: {}",
                            file_path.to_string_lossy()
                        );
                        let mut file_reader = std::io::BufReader::new(file); // Read the existing test file.")
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
                        } else {
                            return Err(anyhow::anyhow!("Error opening test file: {}", e));
                        }
                    }
                }
            };

            if test_start_num >= config.test_gen.test_count {
                println!(
                    "Test file {} is complete. Skipping...",
                    file_path.to_string_lossy()
                );
                continue;
            }

            let test_count = get_test_count(config, opcode);
            for test_num in test_start_num..test_count {
                // Create unique instruction and initial register set for each test.
                // These should not change regardless of test attempt count.

                let mut gen_num: usize = 0;

                // Generate a new random instruction.
                let mut test_instruction = TestInstruction::new(
                    &config.test_gen,
                    opcode,
                    have_group_ext.then_some(opcode_ext),
                    file_seed,
                    test_num,
                    gen_num,
                )?;
                let mut test_registers =
                    TestRegisters::new(context, &config, file_seed, test_num, gen_num);

                // Set flow control end condition
                if config.test_gen.flow_control_opcodes.contains(&opcode) {
                    let flags = context.client.get_flags()?;
                    if flags & ServerFlags::HALT_AFTER_JUMP == 0 {
                        // Enable halt after jump if not already set.
                        context
                            .client
                            .set_flags(flags | ServerFlags::HALT_AFTER_JUMP)?;
                        log::debug!("Enabled HALT_AFTER_JUMP for opcode {:02X}", opcode);
                    }
                }

                let mut test_attempt_ct = 0;
                let mut test_result = generate_test(
                    context,
                    config,
                    file_seed,
                    test_num,
                    gen_num,
                    opcode,
                    have_group_ext.then_some(opcode_ext),
                    &test_instruction,
                    &mut test_registers,
                );

                while !context.dry_run && test_result.is_err() {
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
                            trace_log!(
                                context,
                                "Retrying with new instruction generation: {}",
                                gen_num
                            );
                        } else {
                            trace_error!(
                                context,
                                "Max generation attempts reached for test number {}",
                                test_num
                            );
                        }

                        // Generate a new random instruction.
                        test_instruction = TestInstruction::new(
                            &config.test_gen,
                            opcode,
                            have_group_ext.then_some(opcode_ext),
                            file_seed,
                            test_num,
                            gen_num,
                        )?;
                        test_registers =
                            TestRegisters::new(context, &config, file_seed, test_num, gen_num);
                    }

                    test_result = generate_test(
                        context,
                        config,
                        file_seed,
                        test_num,
                        gen_num,
                        opcode,
                        have_group_ext.then_some(opcode_ext),
                        &test_instruction,
                        &mut test_registers,
                    );

                    if context.dry_run {
                        break;
                    }
                }

                if !context.dry_run {
                    // Add the test to the test file.
                    test_file.add_test(test_result?);
                }
            }
            // Test generation is complete.
            // Log time taken
            context.gen_stop = Instant::now();
            if config.test_exec.show_gen_time {
                let gen_duration = context.gen_stop.duration_since(context.gen_start);

                println!(
                    "Generated {} tests in {:.2?} seconds ({} tests per second)",
                    config.test_gen.test_count,
                    gen_duration,
                    config.test_gen.test_count as f64 / gen_duration.as_secs_f64()
                );
            }

            // Open the file as a Writer.
            log::debug!("Writing test file: {}", file_path.to_string_lossy());

            let file = std::fs::File::create(&file_path)?;
            let mut writer = std::io::BufWriter::new(file);

            test_file.write(&mut writer)?;
        }
    }

    println!(
        "Test generation complete at terminating opcode: {:02X}",
        last_opcode
    );

    Ok(())
}

pub fn get_test_count(config: &Config, opcode: u8) -> usize {
    for ct_override in &config.test_gen.count_overrides {
        let [min, max] = &ct_override.opcode_range[..] else {
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

    trace_log!(context, ">>> Generating test {}", instruction_log_string);
    trace_log!(
        context,
        "Op1:{:?} Op2:{:?}",
        test_instruction.op0_kind(),
        test_instruction.op1_kind()
    );

    // trace_log!(
    //     context,
    //     "Sequence bytes: {:02X?}",
    //     test_instruction.sequence_bytes()
    // );

    let moo_registers = MooRegisters1::try_from(&test_registers.regs)
        .expect("Failed to convert registers to MooRegisters1");

    trace_log!(
        context,
        "{}",
        MooRegisters1Printer {
            regs: &moo_registers,
            cpu_type: config.test_gen.cpu_type,
            diff: None,
        }
    );
}

pub fn generate_test(
    context: &mut TestContext,
    config: &Config,
    file_seed: u64,
    test_num: usize,
    _gen_num: usize,
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

    if test_instruction.iced_instruction().has_rep_prefix()
        || test_instruction.iced_instruction().has_repne_prefix()
    {
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
    if Some(test_num) == config.test_exec.serial_debug_test {
        log::debug!("Enabling serial debug for test number {}", test_num);
        context.client.enable_debug(true)?;
    } else {
        context
            .client
            .enable_debug(config.test_exec.serial_debug_default)?;
    }

    let mut rng = rand::rngs::StdRng::seed_from_u64(file_seed);
    let mut test_seed: u64 = 0;
    // Generate test seed.
    for _ in 0..test_num {
        test_seed = rng.random();
    }

    let mut memory_seed = test_seed as u32;

    // Set memory seed.
    context.client.randomize_memory(memory_seed)?;

    // Determine the memory strategy based on the zero and ff chances.
    let strategy_chance: f32 = rng.random();
    let strategy = if strategy_chance < config.test_gen.mem_zero_chance {
        // Use zero memory strategy.
        trace_log!(context, "Using zero memory strategy");
        MemoryStrategy::Zero
    } else if strategy_chance < config.test_gen.mem_zero_chance + config.test_gen.mem_ones_chance {
        // Use ff memory strategy.
        trace_log!(context, "Using ff memory strategy");
        MemoryStrategy::Ones
    } else {
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
    context.client.set_memory(
        test_registers.instruction_address,
        test_instruction.sequence_bytes(),
    )?;

    // Reset cursor before writing to buffer!
    context.load_register_buffer.set_position(0);
    test_registers
        .regs
        .to_buffer(&mut context.load_register_buffer);

    let mut load_attempt_ct = 0;
    // Load the registers onto the Arduino.
    log::trace!(
        "Uploading registers, attempt {}/{}",
        load_attempt_ct + 1,
        config.test_exec.load_retry
    );

    if let Err(e) = context.client.load_registers_from_buf(
        context.register_set_type,
        context.load_register_buffer.get_ref(),
    ) {
        // If the load fails, retry up to `config.test_exec.load_retry` times.
        while load_attempt_ct < config.test_exec.load_retry {
            load_attempt_ct += 1;
            log::trace!(
                "Retrying register upload, attempt {}/{}",
                load_attempt_ct + 1,
                config.test_exec.load_retry
            );
            if context
                .client
                .load_registers_from_buf(
                    context.register_set_type,
                    context.load_register_buffer.get_ref(),
                )
                .is_ok()
            {
                break;
            }
        }
        if load_attempt_ct >= config.test_exec.load_retry {
            bail!(
                "Failed to upload registers after {} attempts: {}",
                load_attempt_ct,
                e
            );
        }
    }

    let mut state = context.client.get_program_state()?;
    // Wait for the program to finish execution.
    while !matches!(
        state,
        ProgramState::StoreDone | ProgramState::Shutdown | ProgramState::Error
    ) {
        // Sleep for a little bit so we're not spamming the Arduino.
        std::thread::sleep(std::time::Duration::from_millis(
            config.test_exec.polling_sleep.into(),
        ));
        state = context.client.get_program_state()?;
    }

    if matches!(state, ProgramState::Error) {
        log::error!(
            "Error executing instruction: {}",
            context.client.get_last_error()?
        );

        context.last_program_state = Some(ProgramState::Error);
        return Err(anyhow::anyhow!(
            "Error executing instruction: {}",
            context.client.get_last_error()?
        ));
    }

    if matches!(state, ProgramState::Shutdown) {
        log::error!(
            "Shutdown executing instruction: {}",
            context.client.get_last_error()?
        );

        context.last_program_state = Some(ProgramState::Shutdown);
        return Err(anyhow::anyhow!(
            "Shutdown executing instruction: {}",
            context.client.get_last_error()?
        ));
    }

    // Read the registers back from the Arduino.
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
        _ => {
            log::error!("Unknown register set type: {}", reg_type);
            bail!("Unknown register set type: {}", reg_type);
        }
    };

    // Read the cycle states from ArduinoX86.
    log::trace!("Reading cycle states from ArduinoX86...");
    let cycle_states = context.client.get_cycle_states()?;
    log::trace!("Got {} cycle states!", cycle_states.len(),);

    let mut my_cycle_vec = Vec::new();

    // Convert cycle states to MooCycleStates.
    let mut moo_cycle_states = Vec::with_capacity(cycle_states.len());
    for cycle_state in &cycle_states {
        let my_cycle = MyServerCycleState::State286(cycle_state.clone());
        my_cycle_vec.push(my_cycle.clone());
        moo_cycle_states.push(MooCycleState::from(my_cycle));
    }

    log_cycle_states(context, &moo_cycle_states);

    // Collect BusOps from cycle states.
    let bus_ops = collect_bus_ops(&my_cycle_vec);
    log::trace!("Got {} bus operations from cycles", bus_ops.len(),);
    log_bus_ops(context, &bus_ops);

    if let Err(e) = validate_bus_ops(
        config,
        &bus_ops,
        &test_registers.regs,
        opcode,
        test_instruction.iced_instruction().mnemonic(),
        test_instruction.op0_kind(),
        test_instruction.op1_kind(),
    ) {
        log::error!("Bus operation validation failed: {}", e);
        trace_log!(context, "Bus operation validation failed: {}", e);
        //return Err(e);
    }

    // Calculate initial memory state from bus operations.
    let initial_state = initial_state_from_ops(
        CpuWidth::from(context.server_cpu),
        test_registers.regs.cs_base(),
        test_registers.regs.ip(),
        test_instruction.sequence_bytes(),
        0,
        &bus_ops,
    )?;

    log::trace!(
        "Got {} initial RAM entries",
        initial_state.initial_ram.len()
    );

    // Calculate final memory state from initial state and bus operations.
    let final_ram = final_state_from_ops(initial_state.initial_state, &bus_ops)?;

    // Create the initial test state.
    let initial_state = create_state(&test_registers.regs, None, &initial_state.initial_ram);
    // Create the final test state.
    let final_state = create_state(&test_registers.regs, Some(&final_regs), &final_ram);

    // Create the test case.
    let test = MooTest::new(
        test_instruction.name().into(),
        test_instruction.sequence_bytes(),
        initial_state,
        final_state,
        &moo_cycle_states,
    );

    Ok(test)
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

pub fn create_state(
    initial_regs: &Registers,
    final_regs: Option<&Registers>,
    ram: &Vec<[u32; 2]>,
) -> MooTestState {
    let initial_reg_init = MooRegisters1Init::from(initial_regs);
    let final_reg_init = final_regs.map(MooRegisters1Init::from);

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
            value: entry[1] as u8,
        });
    }

    let state_type = if final_regs.is_some() {
        MooStateType::Final
    } else {
        MooStateType::Initial
    };

    MooTestState::new(
        state_type,
        &initial_reg_init,
        final_reg_init.as_ref(),
        Vec::new(),
        ram_vec,
    )
}

pub fn collect_bus_ops(cycle_states: &[MyServerCycleState]) -> Vec<BusOp> {
    let mut bus_ops = Vec::new();

    let mut latched_bus_op = None;
    for cycle_state in cycle_states {
        if let Ok(bus_op) = BusOp::try_from(cycle_state) {
            //log::trace!("Collected bus op: {:?}", bus_op);
            latched_bus_op = Some(bus_op);
        } else {
            if let Some(mut latched_bus_op_inner) = latched_bus_op {
                latched_bus_op_inner.data = cycle_state.data_bus();
                bus_ops.push(BusOp::from(latched_bus_op_inner));
                latched_bus_op = None; // Reset the latched bus operation.
            }
        }
    }
    bus_ops
}

pub fn validate_bus_ops(
    config: &Config,
    bus_ops: &[BusOp],
    registers: &Registers,
    opcode: u8,
    mnemonic: Mnemonic,
    op0: OpKind,
    op1: OpKind,
) -> anyhow::Result<()> {
    let has_memory_read = bus_ops.iter().any(|op| op.op_type == BusOpType::MemRead);
    let has_memory_write = bus_ops.iter().any(|op| op.op_type == BusOpType::MemWrite);

    if let OpKind::Memory = op0 {
        if !has_memory_read {
            if matches!(config.test_gen.cpu_type, MooCpuType::Intel80286)
                && config.test_gen.esc_opcodes.contains(&opcode)
            {
                // 80286 ESC instructions do not automatically read memory.
            } else if !matches!(mnemonic, Mnemonic::Mov) {
                // Mov just overwrites its operand, so we don't need a read.
                return Err(anyhow::anyhow!(
                    "Expected memory read operation for Op0, but none found."
                ));
            }
        }

        if !has_memory_write {
            if matches!(config.test_gen.cpu_type, MooCpuType::Intel80286)
                && config.test_gen.esc_opcodes.contains(&opcode)
            {
                // Okay
            } else {
                match mnemonic {
                    Mnemonic::Test
                    | Mnemonic::Cmp
                    | Mnemonic::Xlatb
                    | Mnemonic::Mul
                    | Mnemonic::Imul
                    | Mnemonic::Div
                    | Mnemonic::Idiv => {
                        // These mnemonics have a memory operand0 without a write operation.
                    }
                    Mnemonic::Rcl
                    | Mnemonic::Rcr
                    | Mnemonic::Shl
                    | Mnemonic::Shr
                    | Mnemonic::Sal
                    | Mnemonic::Sar
                    | Mnemonic::Rol
                    | Mnemonic::Ror => {
                        // If masked cx is 0, these instructions won't write to memory.
                        if config.test_gen.writeless_null_shifts
                            && (registers.cx() & config.test_gen.shift_mask == 0)
                        {
                            // Ok
                        } else {
                            return Err(anyhow::anyhow!(
                                "Expected memory write operation for Op0, but none found."
                            ));
                        }
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Expected memory write operation for Op0, but none found."
                        ));
                    }
                }
            }
        }
    }

    if let OpKind::Memory = op1 {
        if !has_memory_read {
            if !matches!(mnemonic, Mnemonic::Lea) {
                return Err(anyhow::anyhow!(
                    "Expected memory read operation for Op1, but none found."
                ));
            }
        }
    }

    Ok(())
}
