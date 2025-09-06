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

use std::{ffi::OsString, io::BufWriter};

use crate::{
    gen_regs::TestRegisters,
    gen_tests::{compare_registers, generate_test, get_group_extension_range, write_initial_mem},
    instruction::TestInstruction,
    trace_error,
    trace_log,
    Config,
    InstructionSize,
    TestContext,
};
use anyhow::{bail, Context};
use arduinox86_client::ServerFlags;
use moo::prelude::MooTestFile;

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
