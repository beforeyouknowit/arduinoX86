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
use std::collections::VecDeque;
use std::ops::Range;

use rand::prelude::{IndexedRandom, StdRng};
use rand::{Rng, SeedableRng};
use rand_distr::{Beta, Distribution};

use crate::{TerminationCondition, TestGen};

use anyhow::bail;
use iced_x86::{Decoder, DecoderOptions, OpKind};
use moo::types::MooCpuType;

pub struct TestInstruction {
    name: String,
    bytes: Vec<u8>,
    instr_range: Range<usize>,
    sequence_range: Range<usize>,
    prefix_range: Range<usize>,
    iced_i: iced_x86::Instruction,
    op0_kind: OpKind,
    op1_kind: OpKind,
}

impl TestInstruction {
    // Generate a new, random instruction.
    pub fn new(
        config: &TestGen,
        opcode: u8,
        base_seed: u64,
        test_num: usize,
    ) -> anyhow::Result<Self> {
        // Create a new rng seeded by the base seed XOR test number for repeatability.
        let mut rng = StdRng::seed_from_u64(base_seed ^ (test_num as u64));

        let mut instruction_bytes = VecDeque::new();

        // Check opcode is valid.
        if config.excluded_opcodes.contains(&opcode) {
            bail!("Opcode {} is excluded from generation", opcode);
        }

        if config.prefixes.contains(&opcode) {
            bail!("Opcode {} is a prefix and cannot be generated", opcode);
        }

        // Of course we need the opcode itself...
        instruction_bytes.push_back(opcode);

        // Generate a random modrm.
        let modrm = rng.random();

        // We can do specific filters on modrm values here if needed.
        match config.cpu_type {
            MooCpuType::Intel80286 => {
                // Any modrm is fine for the 80286 as invalid forms will generate a UD exception
                // instead of freaking out.
            }
            _ => {
                unimplemented!("Opcode generation for CPU type {:?}", config.cpu_type);
            }
        }

        // Push the modrm byte and six random bytes.
        instruction_bytes.push_back(modrm);
        for _ in 0..6 {
            let byte = rng.random();
            instruction_bytes.push_back(byte);
        }

        // Roll for prefix count.
        // Create a beta distribution to determine the number of prefixes.
        let mut reg_beta = Beta::new(config.prefix_beta[0], config.prefix_beta[1])
            .expect("Invalid beta parameters");

        let beta_out = reg_beta.sample(&mut rng);
        let mut prefix_ct = (beta_out * config.max_prefixes as f64).round() as usize;

        // Set prefix count to zero if opcode is on the list of opcodes excluded from segment prefixes.
        if config.disable_seg_overrides.contains(&opcode) {
            prefix_ct = 0;
        };

        // Add segment override prefixes.
        for i in 0..prefix_ct {
            let segment_prefix = config
                .segment_prefixes
                .choose(&mut rng)
                .ok_or_else(|| anyhow::anyhow!("No segment prefixes defined!"))?;
            instruction_bytes.push_front(*segment_prefix);
        }

        if !config.disable_lock_prefix.contains(&opcode) {
            // Roll for lock prefix chance.
            let lock_prefix_chance = rng.random_range(0.0..1.0);
            if lock_prefix_chance < config.lock_prefix_chance {
                if prefix_ct > 0 {
                    // Replace one of the prefixes with a lock prefix.
                    // Roll for which prefix to replace.
                    let replace_index = rng.random_range(0..prefix_ct);
                    log::trace!(
                        "Replacing prefix at index {} with LOCK prefix",
                        replace_index
                    );
                    // Replace the prefix at the chosen index with the lock prefix.
                    instruction_bytes[replace_index] = config.lock_prefix_opcode;
                } else {
                    instruction_bytes.push_front(config.lock_prefix_opcode);
                    prefix_ct += 1;
                }
            }
        }

        let mut instruction_bytes: Vec<u8> = instruction_bytes.into();

        let mut decoder_opts = DecoderOptions::NO_INVALID_CHECK;
        if matches!(config.cpu_type, MooCpuType::Intel80286) {
            decoder_opts |= DecoderOptions::LOADALL286;
        }
        let mut decoder = Decoder::new(config.cpu_type.bitness(), &instruction_bytes, decoder_opts);

        let iced_i = decoder.decode();
        let instruction_byte_ct = iced_i.len();
        let mut sequence_bytes = instruction_byte_ct;
        let instr_text = iced_i.to_string();

        let op0_kind = iced_i.op0_kind();
        let op1_kind = iced_i.op1_kind();

        if matches!(config.termination_condition, TerminationCondition::Halt) {
            // Insert a HALT instruction at the end of the sequence.
            if instruction_byte_ct == instruction_bytes.len() {
                log::trace!("Appending HALT instruction");
                // Decoded instruction uses all available bytes, so push a new HALT opcode.
                sequence_bytes += 1;
                instruction_bytes.push(0xF4); // HALT instruction for Intel 8086/8088.
            } else if instruction_byte_ct < instruction_bytes.len() {
                log::trace!(
                    "Injecting HALT instruction at offset {}",
                    instruction_byte_ct
                );
                sequence_bytes += 1;
                // Decoded bytes are less than instruction bytes, insert HALT opcode inline.
                instruction_bytes[instruction_byte_ct] = 0xF4; // HALT instruction for Intel 8086/8088.
            } else {
                // Bad condition
                bail!(
                    "Invalid instruction length: {} for opcode {:02X}",
                    instruction_byte_ct,
                    opcode
                );
            }
        }

        Ok(TestInstruction {
            name: instr_text,
            bytes: instruction_bytes,
            instr_range: Range {
                start: 0,
                end: instruction_byte_ct,
            },
            sequence_range: Range {
                start: 0,
                end: sequence_bytes,
            },
            prefix_range: Range {
                start: 0,
                end: prefix_ct,
            },
            iced_i,
            op0_kind,
            op1_kind,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn sequence_bytes(&self) -> &[u8] {
        &self.bytes[self.sequence_range.start..self.sequence_range.end]
    }

    pub fn instr_bytes(&self) -> &[u8] {
        &self.bytes[self.instr_range.start..self.instr_range.end]
    }

    pub fn prefix_bytes(&self) -> &[u8] {
        &self.bytes[self.prefix_range.start..self.prefix_range.end]
    }

    pub fn iced_instruction(&self) -> &iced_x86::Instruction {
        &self.iced_i
    }

    pub fn op0_kind(&self) -> OpKind {
        self.op0_kind
    }

    pub fn op1_kind(&self) -> OpKind {
        self.op1_kind
    }
}
