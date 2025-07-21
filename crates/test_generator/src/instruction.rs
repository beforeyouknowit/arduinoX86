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
use std::{collections::VecDeque, ops::Range};

use rand::{
    prelude::{IndexedRandom, StdRng},
    Rng,
    SeedableRng,
};
use rand_distr::{Beta, Distribution};

use crate::{trace_log, TerminationCondition, TestContext, TestGen};

use crate::gen_regs::TestRegisters;
use anyhow::bail;
use iced_x86::{Decoder, DecoderOptions, Formatter, NasmFormatter, OpKind};
use moo::types::MooCpuType;

pub struct TestInstruction {
    name: String,
    opcode: u8,
    bytes: Vec<u8>,
    test_seed: u64,
    instr_range: Range<usize>,
    sequence_range: Range<usize>,
    prefix_range: Range<usize>,
    iced_i: iced_x86::Instruction,
    mnemonic: String,
    op0_kind: OpKind,
    op1_kind: OpKind,
}

// Create a TestInstruction from a byte slice, such as the bytes chunk array - this allows us
// to create a TestInstruction from an existing test, ie, for validation.
impl From<&[u8]> for TestInstruction {
    fn from(bytes: &[u8]) -> Self {
        let iced_i = Decoder::new(16, bytes, DecoderOptions::NO_INVALID_CHECK).decode();
        let instr_range = 0..iced_i.len();
        let sequence_range = 0..bytes.len();
        let prefix_range = 0..0; // Ignore prefixes for now.
        let mut mnemonic_string = String::new();

        let mut formatter = NasmFormatter::new();
        formatter.format_mnemonic_options(
            &iced_i,
            &mut mnemonic_string,
            iced_x86::FormatMnemonicOptions::NO_PREFIXES,
        );

        TestInstruction {
            name: iced_i.to_string(),
            opcode: bytes[0],
            bytes: bytes.to_vec(),
            test_seed: 0, // No seed for static instructions
            instr_range,
            sequence_range,
            prefix_range,
            iced_i,
            mnemonic: mnemonic_string,
            op0_kind: iced_i.op0_kind(),
            op1_kind: iced_i.op1_kind(),
        }
    }
}

impl TestInstruction {
    // Generate a new, random instruction.
    pub fn new(
        context: &mut TestContext,
        config: &TestGen,
        opcode: u8,
        opcode_ext: Option<u8>,
        test_registers: &TestRegisters,
        test_num: usize,
        gen_number: usize,
    ) -> anyhow::Result<Self> {
        // Put the gen_number into the top 8 bits of the test seed.
        // This allows us to generate tests based off the test number and gen count together.
        let test_seed = context.file_seed ^ ((test_num as u64) | ((gen_number as u64) << 24));

        // Create a new rng seeded by the base seed XOR test seed for repeatability.
        let mut rng = StdRng::seed_from_u64(test_seed);

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
        let mut modrm = rng.random();
        // If the opcode has an extension, set it in the modrm reg field.
        modrm = if let Some(ext) = opcode_ext {
            // Set the reg field of the modrm to the extension value.
            (modrm & 0b1100_0111) | ((ext & 0x07) << 3)
        }
        else {
            modrm
        };

        // Check for modrm overrides.
        for mod_override in &config.modrm_overrides {
            if mod_override.opcode == opcode {
                // Apply the specified modrm mask unless 'invalid_chance' is rolled.
                let valid_chance: f32 = rng.random();
                if valid_chance > mod_override.invalid_chance {
                    // Apply the modrm mask.
                    trace_log!(context, "Applying modrm override for opcode {:02X}", opcode);
                    modrm &= mod_override.mask;
                }
            }
        }

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
        let mut reg_beta = Beta::new(config.prefix_beta[0], config.prefix_beta[1]).expect("Invalid beta parameters");

        let beta_out = reg_beta.sample(&mut rng);
        let mut prefix_ct = (beta_out * config.max_prefixes as f64).round() as usize;

        // Set prefix count to zero if opcode is on the list of opcodes excluded from segment prefixes.
        if config.disable_seg_overrides.contains(&opcode) {
            prefix_ct = 0;
        };

        // Add segment override prefixes.
        for _i in 0..prefix_ct {
            let segment_prefix = config
                .segment_prefixes
                .choose(&mut rng)
                .ok_or_else(|| anyhow::anyhow!("No segment prefixes defined!"))?;
            instruction_bytes.push_front(*segment_prefix);
        }

        if !config.disable_lock_prefix.contains(&opcode) {
            // Roll for lock prefix chance.
            let lock_prefix_roll = rng.random_range(0.0..1.0);
            if lock_prefix_roll < config.lock_prefix_chance {
                if prefix_ct > 0 {
                    // Replace one of the prefixes with a lock prefix.
                    // Roll for which prefix to replace.
                    let replace_index = rng.random_range(0..prefix_ct);
                    log::trace!("Replacing prefix at index {} with LOCK prefix", replace_index);
                    // Replace the prefix at the chosen index with the lock prefix.
                    instruction_bytes[replace_index] = config.lock_prefix_opcode;
                }
                else {
                    instruction_bytes.push_front(config.lock_prefix_opcode);
                    prefix_ct += 1;
                }
            }
        }

        if config.rep_opcodes.contains(&opcode) {
            // Roll for REP prefix chance.
            let rep_prefix_roll = rng.random_range(0.0..1.0);
            if rep_prefix_roll < config.rep_prefix_chance {
                if prefix_ct > 0 {
                    // Replace one of the prefixes with a REP prefix.
                    // Roll for which prefix to replace.
                    let replace_index = rng.random_range(0..prefix_ct);
                    log::trace!("Replacing prefix at index {} with REP prefix", replace_index);
                    // Replace the prefix at the chosen index with the REP prefix.
                    instruction_bytes[replace_index] = *config.rep_prefixes.choose(&mut rng).unwrap();
                }
                else {
                    instruction_bytes.push_front(*config.rep_prefixes.choose(&mut rng).unwrap());
                    prefix_ct += 1;
                }
            }
        }

        let mut instruction_bytes: Vec<u8> = instruction_bytes.into();

        let mut decoder_opts = DecoderOptions::NO_INVALID_CHECK;
        if matches!(config.cpu_type, MooCpuType::Intel80286) {
            decoder_opts |= DecoderOptions::LOADALL286;
        }

        let initial_decode_buffer = instruction_bytes.clone();
        let mut decoder = Decoder::new(config.cpu_type.bitness(), &initial_decode_buffer, decoder_opts);

        let mut iced_i = decoder.decode();
        let mut instruction_byte_ct = iced_i.len();
        let mut sequence_bytes = instruction_byte_ct;
        let mut instr_text = iced_i.to_string();

        let op0_kind = iced_i.op0_kind();
        let op1_kind = iced_i.op1_kind();

        // Modify instruction with iced if necessary.
        let mut modified_iced = false;
        match op0_kind {
            OpKind::NearBranch16 => {
                let mut branch_val = iced_i.near_branch16();
                trace_log!(context, "Near branch value: {:04X}", branch_val);
                if branch_val == config.near_branch_ban {
                    while branch_val == config.near_branch_ban {
                        trace_log!(context, "Near branch with banned value!");
                        branch_val = rng.random::<i8>() as u16;
                    }
                    log::trace!("Setting near branch value to {:04X}", branch_val);
                    iced_i.set_near_branch16(branch_val);
                    modified_iced = true;
                }
            }
            _ => {}
        }

        match op1_kind {
            OpKind::Immediate8 => {
                // iced considers rcl reg, 1 as an immediate8, and it is an error to override it
                // so only override the immediate if it is not 1.
                if iced_i.immediate8() != 0x01 {
                    // Roll for immediate override.
                    let immediate_roll = rng.random_range(0.0..1.0);
                    if immediate_roll < config.imm_zero_chance {
                        trace_log!(context, "Overriding immediate to zero");
                        iced_i.set_immediate8(0x00);
                        modified_iced = true;
                    }
                    else if immediate_roll < config.imm_zero_chance + config.imm_ones_chance {
                        trace_log!(context, "Overriding immediate to all-ones");
                        iced_i.set_immediate8(0xFF);
                        modified_iced = true;
                    }
                }
            }
            OpKind::Immediate8to16 => {
                // Roll for immediate override.
                let immediate_roll = rng.random_range(0.0..1.0);
                if immediate_roll < config.imm_zero_chance {
                    trace_log!(context, "Overriding immediate8s to zero");
                    iced_i.set_immediate8to16(0x0000);
                    modified_iced = true;
                }
                else if immediate_roll < config.imm_zero_chance + config.imm8s_min_chance {
                    trace_log!(context, "Overriding immediate8s to minimum");
                    iced_i.set_immediate8to16(i16::MIN);
                    modified_iced = true;
                }
                else if immediate_roll < config.imm_zero_chance + config.imm8s_min_chance + config.imm8s_max_chance {
                    trace_log!(context, "Overriding immediate8s to maximum");
                    iced_i.set_immediate8to16(i16::MAX);
                    modified_iced = true;
                }
            }
            OpKind::Immediate16 => {
                // Roll for immediate override.
                let immediate_roll = rng.random_range(0.0..1.0);
                if immediate_roll < config.imm_zero_chance {
                    trace_log!(context, "Overriding immediate to zero");
                    iced_i.set_immediate16(0x0000);
                    modified_iced = true;
                }
                else if immediate_roll < config.imm_zero_chance + config.imm_ones_chance {
                    trace_log!(context, "Overriding immediate to all-ones");
                    iced_i.set_immediate16(0xFFFF);
                    modified_iced = true;
                }
            }
            _ => {}
        }

        if modified_iced {
            let mut encoder = iced_x86::Encoder::new(config.cpu_type.bitness());
            encoder.encode(&iced_i, 0)?;
            let buffer = encoder.take_buffer();

            // Iced will not encode multiple prefixes. If we generated multiple prefixes for this
            // instruction, it would be a pain to try to copy the iced-encoded instruction at the
            // correct spot, so instead we'll just replace the entire instruction bytes vector
            // with the new bytes. This means that we have a maximum of one segment override
            // prefix whenever we override an immediate, but this is an acceptable limitation.

            instruction_bytes = buffer.to_vec();
            decoder = Decoder::new(config.cpu_type.bitness(), &instruction_bytes, decoder_opts);
            iced_i = decoder.decode();
            instr_text = iced_i.to_string();
            instruction_byte_ct = iced_i.len();
            sequence_bytes = instruction_byte_ct;
            trace_log!(
                context,
                "New instruction bytes: {:X?} ct:{}",
                instruction_bytes,
                instruction_byte_ct
            );
        }

        if matches!(config.termination_condition, TerminationCondition::Halt) {
            // Insert a HALT instruction at the end of the sequence.
            if instruction_byte_ct == instruction_bytes.len() {
                log::trace!("Appending HALT instruction");
                // Decoded instruction uses all available bytes, so push a new HALT opcode.
                sequence_bytes += 1;
                instruction_bytes.push(0xF4); // HALT instruction for Intel 8086/8088.
            }
            else if instruction_byte_ct < instruction_bytes.len() {
                log::trace!("Injecting HALT instruction at offset {}", instruction_byte_ct);
                sequence_bytes += 1;
                // Decoded bytes are less than instruction bytes, insert HALT opcode inline.
                instruction_bytes[instruction_byte_ct] = 0xF4; // HALT instruction for Intel 8086/8088.
            }
            else {
                // Bad condition
                bail!(
                    "Invalid instruction length: {} for opcode {:02X}",
                    instruction_byte_ct,
                    opcode
                );
            }
        }

        let mut mnemonic_string = String::new();

        let mut formatter = NasmFormatter::new();
        formatter.format_mnemonic_options(
            &iced_i,
            &mut mnemonic_string,
            iced_x86::FormatMnemonicOptions::NO_PREFIXES,
        );

        Ok(TestInstruction {
            name: instr_text,
            opcode,
            bytes: instruction_bytes,
            test_seed,
            instr_range: Range {
                start: 0,
                end:   instruction_byte_ct,
            },
            sequence_range: Range {
                start: 0,
                end:   sequence_bytes,
            },
            prefix_range: Range {
                start: 0,
                end:   prefix_ct,
            },
            mnemonic: mnemonic_string,
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

    pub fn mnemonic(&self) -> &str {
        &self.mnemonic
    }
}
