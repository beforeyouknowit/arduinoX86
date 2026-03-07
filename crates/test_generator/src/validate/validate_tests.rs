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

use crate::{
    batch::TestCandidateList,
    generate::{gen_regs::TestRegisters, gen_test::generate_test, generation_stats::GenerationStats},
    instruction::instruction::TestInstruction,
    registers::compare_registers,
    trace_error,
    trace_log,
    validate::{memory::write_initial_mem, validate_test::validate_test},
    TestContext,
};
use anyhow::{bail, Context};
use arduinox86_client::ServerFlags;
use marty_dasm::Opcode;
use moo::{prelude::*, types::MooCpuType};
use std::{ffi::OsString, io::BufWriter, time::Instant};

pub fn validate_tests(ctx: &mut TestContext) -> anyhow::Result<()> {
    ctx.gen_ct = 0;
    ctx.gen_start = Instant::now();

    let config = ctx.cfg.clone();

    // Set the opcode range to generate as specified by the configuration.
    // `opcode_range` is now a two-element array so safe to index directly.
    let mut opcode_range_start = config.test_gen.opcode_range[0];
    let mut opcode_range_end = config.test_gen.opcode_range[1];

    if let Some(opcode_override) = config.test_gen.opcode_override {
        opcode_range_start = opcode_override;
        opcode_range_end = opcode_override;
    }

    println!(
        "Validating tests for opcodes from [{} to {}]",
        Opcode::from(opcode_range_start),
        Opcode::from(opcode_range_end)
    );

    // Tell ArduinoX86 to execute instructions automatically.
    let mut server_flags =
        ServerFlags::EXECUTE_AUTOMATIC | ServerFlags::ENABLE_CYCLE_LOGGING | ServerFlags::USE_SDRAM_BACKEND;

    // Enable SMM register loading if we have a 386 CPU.
    // We can load registers as we exit SMM between tests - this avoids the lengthy 386EX setup
    // routine to disable wait states.
    if let MooCpuType::Intel80386Ex = config.test_gen.cpu_type {
        server_flags |= ServerFlags::USE_SMM;
    }

    ctx.client.set_flags(server_flags)?;
    ctx.client.enable_debug(config.test_exec.serial_debug_default)?;

    let mut generation_job = TestCandidateList::collect(ctx);
    if config.test_gen.skip_validated {
        generation_job.filter_validated(ctx);
    }
    if generation_job.is_empty() {
        bail!("No work to do based on filtering and batch criteria.");
    }
    generation_job.log(ctx);
    ctx.gen_total = generation_job.test_ct(ctx);

    let mut last_opcode: u16 = generation_job.iter().next().unwrap().opcode.into();

    'validateLoop: for test_gen in generation_job.iter() {
        ctx.file_gen_ct = 0;
        ctx.stats = GenerationStats::default();
        ctx.stats.mnemonic_set.clear();
        ctx.test_opcode_size_prefix = test_gen.size_prefix;

        let opcode_raw: u16 = test_gen.opcode.into();
        last_opcode = opcode_raw;

        // Create the output file path.
        let mut file_path = ctx.output_path.clone();
        let filename = test_gen.filename();
        file_path.push(filename.clone());

        // Create the trace file.
        let trace_filename = test_gen.trace_filename(&config.test_gen.trace_file_suffix);
        let trace_file_path = ctx.trace_path.join(trace_filename);
        let trace_file = std::fs::File::create(&trace_file_path)
            .with_context(|| format!("Creating trace file: {}", trace_file_path.display()))?;

        ctx.trace_log = BufWriter::new(trace_file);

        // Open `file_path` for reading as a BufReader.
        let test_file = match std::fs::File::open(&file_path) {
            Ok(file) => {
                let mut file_reader = std::io::BufReader::new(file);
                let test_file = MooTestFile::read(&mut file_reader)?;

                println!(
                    "Read {} tests from file: {}",
                    test_file.test_ct(),
                    file_path.to_string_lossy()
                );
                test_file
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    log::debug!("File {} not found, skipping...", file_path.to_string_lossy());
                    continue;
                }
                else {
                    return Err(anyhow::anyhow!("Error opening test file: {}", e));
                }
            }
        };

        let metadata = match test_file.metadata() {
            Some(meta) => meta,
            None => {
                return Err(anyhow::anyhow!(
                    "Test file {} has no metadata.",
                    file_path.to_string_lossy()
                ));
            }
        };

        // Validate the tests.
        for (test_num, test_in) in test_file.tests().iter().enumerate() {
            //log::debug!("Initial test file registers: {:#X?}", test_in.initial_regs());
            //log::debug!("Final test file registers: {:#X?}", test_in.final_regs());

            let gen_num: usize = 0;

            let opcode = (metadata.opcode as u16).into();

            if test_in.bytes().is_empty() {
                let err_str = format!(
                    "Test number {} in file {} has no instruction bytes!",
                    test_num,
                    file_path.to_string_lossy()
                );
                trace_error!(ctx, "{err_str}");
                bail!("{err_str}");
            }

            let instruction_bytes = test_in.bytes();

            let test_registers = TestRegisters::from(test_in.initial_state().regs());

            //println!("Test registers: {:#X?}", test_registers.regs);

            let test_instruction =
                match TestInstruction::load(ctx, opcode, test_in.name(), instruction_bytes, &test_registers.regs) {
                    Ok(inst) => inst,
                    Err(e) => {
                        let err_str = format!(
                            "Failed to load instruction for opcode {} at test number {}: {}",
                            opcode, test_num, e
                        );
                        trace_error!(ctx, "{err_str}");
                        if ctx.cfg.test_exec.validate_mode.stop_on_error {
                            bail!("{err_str}");
                        }
                        else {
                            continue 'validateLoop;
                        }
                    }
                };

            if test_instruction.bytes().len() != test_in.bytes().len() {
                let err_str = format!(
                    "Instruction byte length mismatch for opcode {} at test number {}: expected {}, got {}",
                    opcode,
                    test_num,
                    test_in.bytes().len(),
                    test_instruction.bytes().len()
                );
                trace_error!(ctx, "{err_str}");
                if ctx.cfg.test_exec.validate_mode.stop_on_error {
                    bail!("{err_str}");
                }
                else {
                    continue 'validateLoop;
                }
            }

            //println!("Got bytes for test: {:X?}", test_instruction.bytes());

            // Write initial memory state to device.
            let initial_mem = test_in.initial_state().ram();

            write_initial_mem(ctx, &initial_mem)?;

            // Set flow control end condition
            if ctx.cfg.test_gen.flow_control_opcodes.contains(&opcode.into()) {
                let flags = ctx.client.get_flags()?;
                if flags & ServerFlags::HALT_AFTER_JUMP == 0 {
                    // Enable halt after jump if not already set.
                    ctx.client.set_flags(flags | ServerFlags::HALT_AFTER_JUMP)?;
                    log::debug!("Enabled HALT_AFTER_JUMP for opcode {}", opcode);
                }
            }

            let mut test_attempt_ct = 0;
            let mut test_validated = false;
            let mut test_result;

            'attemptLoop: while test_attempt_ct < ctx.cfg.test_exec.validate_mode.attempts {
                if test_attempt_ct > 0 {
                    trace_log!(
                        ctx,
                        "Retrying test for opcode {} at test number {}, attempt {}/{}...",
                        opcode,
                        test_num,
                        test_attempt_ct + 1,
                        ctx.cfg.test_exec.validate_mode.attempts
                    );
                }
                test_attempt_ct += 1;
                test_result = validate_test(
                    ctx,
                    test_num,
                    gen_num,
                    opcode,
                    test_gen.opcode_extension,
                    &test_instruction,
                    &test_registers,
                );
                match test_result {
                    Ok(test_out) => match post_test_validation(ctx, opcode, test_num as u32, test_in, &test_out) {
                        Ok(_) => {
                            trace_log!(
                                ctx,
                                "{}:{:05X} test successfully validated, attempt {}/{}",
                                opcode,
                                test_num,
                                test_attempt_ct,
                                ctx.cfg.test_exec.validate_mode.attempts
                            );
                            ctx.gen_ct += 1;
                            test_validated = true;
                            break 'attemptLoop;
                        }
                        Err(e) => {
                            let err_str = format!(
                                "Post-test validation failed for opcode {} at test number {}, attempt {}/{}: {}",
                                opcode, test_num, test_attempt_ct, ctx.cfg.test_exec.validate_mode.attempts, e
                            );
                            trace_error!(ctx, "{err_str}");
                            continue 'attemptLoop;
                        }
                    },
                    Err(e) => {
                        let err_str = format!(
                            "Failed to validate test for opcode {} at test number {}, attempt {}/{}, error: {}",
                            opcode, test_num, test_attempt_ct, ctx.cfg.test_exec.test_retry, e
                        );
                        trace_error!(ctx, "{err_str}");
                        continue 'attemptLoop;
                    }
                }
            }

            if !test_validated {
                let err_str = format!(
                    "Test for opcode {} at test number {} failed to validate after {} attempts.",
                    opcode, test_num, ctx.cfg.test_exec.validate_mode.attempts
                );
                trace_error!(ctx, "{err_str}");
                if ctx.cfg.test_exec.validate_mode.stop_on_error {
                    bail!("{err_str}");
                }
                else {
                    continue 'validateLoop;
                }
            }
        }

        // If validation passed, move the test to the validated directory if the option is set.
        if ctx.cfg.test_exec.validate_mode.move_after_validate {
            let mut validated_path = ctx.validate_output_path.clone();
            validated_path.push(filename.clone());

            std::fs::rename(&file_path, &validated_path).with_context(|| {
                format!(
                    "Moving validated file from {} to {}",
                    file_path.to_string_lossy(),
                    validated_path.to_string_lossy()
                )
            })?;
            log::debug!(
                "Moved validated file from {} to {}",
                file_path.to_string_lossy(),
                validated_path.to_string_lossy()
            );
        }
    }

    Ok(())
}

pub fn post_test_validation(
    ctx: &mut TestContext,
    opcode: Opcode,
    test_num: u32,
    test_in: &MooTest,
    test_out: &MooTest,
) -> anyhow::Result<()> {
    // Compute the register delta.
    let delta_regs = test_in.initial_state().regs().delta(&test_out.final_state().regs());

    if delta_regs != *test_in.final_state().regs() {
        trace_error!(
            ctx,
            "Register mismatch for opcode {} at test number {}!",
            opcode,
            test_num,
        );

        log::debug!("Computed delta registers from output test: {:#X?}", delta_regs);
        log::debug!("Final registers from input test: {:#X?}", test_in.final_state().regs());

        compare_registers(&test_out.final_state().regs(), test_in.final_state().regs());

        let err_str = format!("Register mismatch for opcode {} at test number {}", opcode, test_num);
        trace_error!(ctx, "{err_str}");
        bail!("{err_str}");
    }
    trace_log!(ctx, "{}:{:05X} registers validated.", opcode, test_num);

    if test_in.final_state().ram().len() != test_out.final_state().ram().len() {
        let err_str = format!(
            "Memory state entry count mismatch for opcode {} at test number {}: expected {}, got {}",
            opcode,
            test_num,
            test_in.final_state().ram().len(),
            test_out.final_state().ram().len()
        );
        trace_error!(ctx, "{err_str}");
        bail!("{err_str}");
    }

    for (in_mem, out_mem) in test_out
        .final_state()
        .ram()
        .iter()
        .zip(test_in.final_state().ram().iter())
    {
        if in_mem.address != out_mem.address || in_mem.value != out_mem.value {
            let err_str = format!("Memory state mismatch for opcode {} at test number {}: expected address {:X} value {:02X}, got address {:X} value {:02X}",
                                  opcode,
                                  test_num,
                                  out_mem.address,
                                  out_mem.value,
                                  in_mem.address,
                                  in_mem.value
            );
            trace_error!(ctx, "{err_str}");
            bail!("{err_str}");
        }
    }

    // Compare cycle states
    if test_in.cycles().len() != test_out.cycles().len() {
        let err_str = format!(
            "Cycle count mismatch for opcode {} at test number {}: expected {}, got {}",
            opcode,
            test_num,
            test_in.cycles().len(),
            test_out.cycles().len()
        );
        trace_log!(ctx, "{err_str}");
        log::warn!("{err_str}");
    }

    Ok(())
}
