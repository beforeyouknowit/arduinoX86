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
use super::{Config, CpuMode, CpuType, Registers, TerminationCondition, TestContext};
use crate::display::{print_regs, print_regs_v2};
use crate::gen_opcode::OpcodeGenerator;
use crate::gen_regs::randomize_v2;
use anyhow::bail;
use ard808x_client::{
    ProgramState, RegisterSetType, RemoteCpuRegistersV1, RemoteCpuRegistersV2, ServerCpuType,
    ServerFlags,
};
use iced_x86::{Decoder, DecoderOptions};
use std::io::Cursor;
use std::ops::Range;
use toml::Value;

pub fn gen_tests(context: &mut TestContext, config: &Config) -> anyhow::Result<()> {
    let mut test_num = 0;
    let mut register_buffer = Cursor::new(Vec::with_capacity(16));
    let mut opcode_range_start: u8 = 0;
    let mut opcode_range_end: u8 = 0xFF;
    let mut register_buf = vec![0u8; 102];

    let server_cpu = ServerCpuType::from(config.test_gen.cpu_type);
    let register_set_type = RegisterSetType::from(server_cpu);

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

    let mut opcode_generator = OpcodeGenerator {
        config: config.test_gen.clone(),
    };

    // Tell ArduinoX86 to execute instructions automatically.
    context.client.set_flags(ServerFlags::EXECUTE_AUTOMATIC)?;

    for opcode in opcode_range_start..=opcode_range_end {
        if config.test_gen.excluded_opcodes.contains(&opcode) {
            continue;
        }
        if config.test_gen.prefixes.contains(&opcode) {
            continue;
        }

        for test_num in 0..config.test_gen.test_count {
            let mut instruction_bytes = opcode_generator.generate(context, opcode, None)?;

            let mut decoder = Decoder::new(
                config.test_gen.cpu_type.bitness(),
                &instruction_bytes,
                DecoderOptions::NO_INVALID_CHECK | DecoderOptions::LOADALL286,
            );

            let instruction = decoder.decode();
            let n_bytes = instruction.len();
            let mut sequence_bytes = n_bytes;
            let instr_text = instruction.to_string();

            log::trace!(
                "Instruction termination condition: {:?}",
                config.test_gen.termination_condition
            );

            if matches!(
                config.test_gen.termination_condition,
                TerminationCondition::Halt
            ) {
                // Insert a HALT instruction at the end of the sequence.
                if n_bytes == instruction_bytes.len() {
                    log::trace!("Appending HALT instruction");
                    // Decoded instruction uses all available bytes, so push a new HALT opcode.
                    sequence_bytes += 1;
                    instruction_bytes.push(0xF4); // HALT instruction for Intel 8086/8088.
                } else if n_bytes < instruction_bytes.len() {
                    log::trace!("Injecting HALT instruction at offset {}", n_bytes);
                    sequence_bytes += 1;
                    // Decoded bytes are less than instruction bytes, insert HALT opcode inline.
                    instruction_bytes[n_bytes] = 0xF4; // HALT instruction for Intel 8086/8088.
                } else {
                    // Bad condition
                    bail!(
                        "Invalid instruction length: {} for opcode {:02X}",
                        n_bytes,
                        opcode
                    );
                }
            }

            // Randomize the registers.
            let instruction_range: Range<u32> = Range {
                start: config.test_gen.instruction_address_range[0],
                end: config.test_gen.instruction_address_range[1],
            };
            let mut instruction_good = false;
            let mut instruction_address = 0;
            let mut regs = Registers::V1(Default::default());

            // Repeatedly generate random registers until one set qualifies as valid.
            while !instruction_good {
                regs = match config.test_gen.cpu_type {
                    CpuType::Intel80286 => {
                        let mut random_v2 = Registers::V2(RemoteCpuRegistersV2::default());
                        randomize_v2(context, config.test_gen.clone(), &mut random_v2);

                        print_regs(&random_v2, config.test_gen.cpu_type.into());
                        random_v2
                    }
                    _ => Registers::V1(RemoteCpuRegistersV1::default()),
                };

                if matches!(config.test_gen.cpu_mode, CpuMode::Real) {
                    // Doing real mode test. Normalize the segment descriptors.
                    regs.normalize_descriptors();
                }

                // Check if the instruction is valid with the current registers.
                instruction_address = regs.calculate_code_address() & config.test_gen.address_mask;
                if instruction_range.contains(&instruction_address) {
                    instruction_good = true;
                }
            }

            // Randomize memory on the Arduino at the specified test interval.
            log::trace!("Randomizing server memory...");
            if (test_num > 0) && (test_num % config.test_gen.randomize_mem_interval == 0) {
                context.client.randomize_memory()?;
            }

            // Upload the instruction sequence.
            log::trace!("Uploading instruction sequence...");
            context
                .client
                .set_memory(instruction_address, &instruction_bytes[..sequence_bytes])?;

            regs.to_buffer(&mut register_buffer);

            // Load the registers onto the Arduino.
            log::trace!("Uploading registers...");
            context
                .client
                .load_registers_from_buf(register_set_type, register_buffer.get_ref())?;

            // Write the instruction into memory.

            println!(
                "{:05} | {:02X} {:<25} │ {:02X?}",
                test_num,
                opcode,
                instr_text,
                &instruction_bytes[..n_bytes]
            );

            let mut state = context.client.get_program_state()?;
            // Wait for the program to finish execution.
            while !matches!(state, ProgramState::StoreDone | ProgramState::Error) {
                // Sleep for a little bit so we're not spamming the Arduino.
                std::thread::sleep(std::time::Duration::from_millis(
                    config.test_exec.polling_sleep.into(),
                ));
                state = context.client.get_program_state()?;
            }

            if matches!(state, ProgramState::Error) {
                log::error!(
                    "Error executing instruction: {:?}",
                    context.client.get_last_error()
                );
            }

            // Read the registers back from the Arduino.
            log::trace!("Reading registers back from Arduino...");
            let reg_type = context
                .client
                .store_registers_to_buf(&mut register_buf)
                .map_err(|e| anyhow::anyhow!("Error reading registers: {}", e))?;

            match reg_type {
                0x0 => {
                    // V1 registers
                }
                0x1 => {
                    // V2 registers
                    let regs_v2 = RemoteCpuRegistersV2::try_from(register_buf.as_slice())
                        .map_err(|e| anyhow::anyhow!("Error parsing V2 registers: {}", e))?;
                    print_regs_v2(&regs_v2, config.test_gen.cpu_type.into());
                }
                _ => {
                    log::error!("Unknown register set type: {}", reg_type);
                    bail!("Unknown register set type: {}", reg_type);
                }
            }
        }
    }

    Ok(())
}
