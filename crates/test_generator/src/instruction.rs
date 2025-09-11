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

use crate::{
    cpu_common::{
        AddressOffset,
        AddressOffset16,
        AddressOffset32,
        AddressingMode,
        AddressingMode16,
        AddressingMode32,
        Displacement,
    },
    gen_regs::TestRegisters,
    modrm::{ModRmByte16, ModRmByte32, SibByte},
    trace_log,
    AddressSize,
    InstructionSize,
    Opcode,
    TerminationCondition,
    TestContext,
    TestGen,
};

use anyhow::bail;
use arduinox86_client::registers_common::SegmentSize;
use iced_x86::{Decoder, DecoderOptions, Formatter, NasmFormatter, OpCodeTableKind, OpKind, Register};
use moo::types::MooCpuType;
use rand::{
    prelude::{IndexedRandom, StdRng},
    Rng,
    SeedableRng,
};
use rand_distr::{Beta, Distribution};

pub struct TestInstruction {
    name: String,
    operand_size: InstructionSize,
    address_size: AddressSize,
    opcode: Opcode,
    bytes: Vec<u8>,
    test_seed: u64,
    instr_range: Range<usize>,
    sequence_range: Range<usize>,
    prefix_range: Range<usize>,
    iced_i: iced_x86::Instruction,
    mnemonic: String,
    op0_kind: OpKind,
    op1_kind: OpKind,
    addressing_mode: Option<AddressingMode>,
    modrm_offset: usize,
    displacement_offset: Option<usize>,
}

// Create a TestInstruction from a byte slice, such as the bytes chunk array - this allows us
// to create a TestInstruction from an existing test, ie, for validation.
impl From<(InstructionSize, AddressSize, &[u8])> for TestInstruction {
    fn from(data: (InstructionSize, AddressSize, &[u8])) -> Self {
        let iced_i = Decoder::new(data.0.into(), data.2, DecoderOptions::NO_INVALID_CHECK).decode();
        let instr_range = 0..iced_i.len();
        let sequence_range = 0..data.2.len();
        let prefix_range = 0..0; // TODO: Ignore prefixes for now.
        let mut mnemonic_string = String::new();

        let mut formatter = NasmFormatter::new();
        formatter.format_mnemonic_options(
            &iced_i,
            &mut mnemonic_string,
            iced_x86::FormatMnemonicOptions::NO_PREFIXES,
        );

        let info = iced_i.op_code();
        let low = info.op_code() as u16; // low opcode byte (xx)

        let opcode = match info.table() {
            OpCodeTableKind::T0F => 0x0F00 | low, // 2-byte opcode:  0F xx  -> 0x0Fxx
            _ => low,
        };

        TestInstruction {
            name: format_instruction(&iced_i),
            operand_size: data.0,
            address_size: data.1,
            opcode: opcode.into(),
            bytes: data.2.to_vec(),
            test_seed: 0, // No seed for static instructions
            instr_range,
            sequence_range,
            prefix_range,
            iced_i,
            mnemonic: mnemonic_string,
            op0_kind: iced_i.op0_kind(),
            op1_kind: iced_i.op1_kind(),
            addressing_mode: None,
            modrm_offset: 0,
            displacement_offset: None,
        }
    }
}

pub fn format_instruction(iced_i: &iced_x86::Instruction) -> String {
    let mut instr_text = String::new();
    let mut formatter = NasmFormatter::new();

    formatter.options_mut().set_always_show_segment_register(true);
    formatter.options_mut().set_add_leading_zero_to_hex_numbers(false);
    formatter.options_mut().set_always_show_scale(true);
    //formatter.options_mut().set_show_zero_displacements(true);

    formatter.format(&iced_i, &mut instr_text);
    instr_text
}

pub fn count_prefixes(bytes: &[u8]) -> usize {
    let mut count = 0;
    for byte in bytes {
        match byte {
            0xF0 | 0xF2 | 0xF3 | 0x2E | 0x36 | 0x3E | 0x26 | 0x64 | 0x65 | 0x66 | 0x67 => count += 1,
            _ => break,
        }
    }
    count
}

pub fn get_effective_segment(iced_i: &iced_x86::Instruction) -> Option<Register> {
    match iced_i.memory_segment() {
        Register::DS | Register::ES | Register::FS | Register::GS | Register::SS | Register::CS => {
            Some(iced_i.memory_segment())
        }
        _ => None,
    }
}

impl TestInstruction {
    // Generate a new, random instruction.
    pub fn new(
        context: &mut TestContext,
        config: &TestGen,
        opcode: Opcode,
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

        let mut instruction_bytes: VecDeque<u8> = VecDeque::new();

        // Check opcode is valid.
        if config.excluded_opcodes.contains(&opcode.into()) {
            bail!("Opcode {} is excluded from generation", opcode);
        }

        if !opcode.is_extended() && config.prefixes.contains(&opcode.into()) {
            bail!("Opcode {} is a prefix and cannot be generated", opcode);
        }

        // Of course we need the opcode itself...
        instruction_bytes.extend(opcode.to_bytes());

        // Generate a random modrm.
        let mut modrm: u8 = rng.random();
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
            if mod_override.opcode == opcode.into() {
                // Apply the specified modrm mask unless 'invalid_chance' is rolled.
                let valid_chance: f32 = rng.random();
                if valid_chance > mod_override.invalid_chance {
                    // Reject register forms if specified.
                    while !mod_override.allow_reg_form && (modrm & 0b1100_0000 == 0b1100_0000) {
                        modrm = rng.random();
                    }

                    // Apply the modrm mask.
                    trace_log!(context, "Applying modrm override for opcode {}", opcode);
                    modrm &= mod_override.mask;
                }
            }
        }

        // We can do specific filters on modrm values here if needed.
        match config.cpu_type {
            MooCpuType::Intel80286 | MooCpuType::Intel80386Ex => {
                // Any modrm is fine for the 80286 as invalid forms will generate a UD exception
                // instead of freaking out.
            }
            _ => {
                unimplemented!("Opcode generation for CPU type {:?}", config.cpu_type);
            }
        }

        // Push the modrm byte and six random bytes.
        instruction_bytes.push_back(modrm);
        let mut modrm_offset = instruction_bytes.len() - 1;
        trace_log!(
            context,
            "instruction bytes: {:X?} Added modrm at offset {}",
            instruction_bytes,
            modrm_offset
        );

        for _ in 0..6 {
            let byte = rng.random();
            instruction_bytes.push_back(byte);
        }

        // Append specified opcode size prefixes.
        for byte in Vec::<u8>::from(context.test_opcode_size_prefix) {
            instruction_bytes.push_front(byte);
            modrm_offset += 1;
        }

        // Roll for prefix count.
        // Create a beta distribution to determine the number of prefixes.
        let mut reg_beta = Beta::new(config.prefix_beta[0], config.prefix_beta[1]).expect("Invalid beta parameters");

        let beta_out = reg_beta.sample(&mut rng);
        let mut prefix_ct = (beta_out * config.max_prefixes as f64).round() as usize;

        // Set prefix count to zero if opcode is on the list of opcodes excluded from segment prefixes.
        if config.disable_seg_overrides.contains(&opcode.into()) {
            prefix_ct = 0;
        };

        // Add segment override prefixes.
        for _i in 0..prefix_ct {
            let segment_prefix = config
                .segment_prefixes
                .choose(&mut rng)
                .ok_or_else(|| anyhow::anyhow!("No segment prefixes defined!"))?;
            instruction_bytes.push_front(*segment_prefix);
            trace_log!(context, "prefix: modrm_offset++");
            modrm_offset += 1;
        }

        if !config.disable_lock_prefix.contains(&opcode.into()) {
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
                    trace_log!(context, "lock: modrm_offset++");
                    modrm_offset += 1;
                    prefix_ct += 1;
                }
            }
        }

        if config.rep_opcodes.contains(&opcode.into()) {
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
                    trace_log!(context, "rep: modrm_offset++");
                    modrm_offset += 1;
                    prefix_ct += 1;
                }
            }
        }

        let mut instruction_bytes: Vec<u8> = instruction_bytes.into();
        trace_log!(context, "final instruction bytes: {:X?}", instruction_bytes);

        let mut decoder_opts = DecoderOptions::NO_INVALID_CHECK;
        if matches!(config.cpu_type, MooCpuType::Intel80286) {
            decoder_opts |= DecoderOptions::LOADALL286;
        }

        let mut decode_buffer = instruction_bytes.clone();
        let mut decoder = Decoder::new(16, &decode_buffer, decoder_opts);

        let mut iced_i = decoder.decode();
        let mut instruction_byte_ct = iced_i.len();
        let mut sequence_bytes = instruction_byte_ct;
        let mut instr_text = format_instruction(&iced_i);

        let prefix_ct = count_prefixes(&instruction_bytes);

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
                        trace_log!(context, "Overriding immediate8 to zero");
                        iced_i.set_immediate8(0x00);
                        modified_iced = true;
                    }
                    else if immediate_roll < config.imm_zero_chance + config.imm_ones_chance {
                        trace_log!(context, "Overriding immediate8 to all-ones");
                        iced_i.set_immediate8(0xFF);
                        modified_iced = true;
                    }
                    else if immediate_roll < config.imm_inject_chance {
                        let index = rng.random_range(0..config.inject_values.len());
                        let inject_value = config.inject_values[index] as u8;
                        trace_log!(context, "Injecting immediate8 value {:02X}", inject_value);
                        iced_i.set_immediate8(inject_value);
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
                else if immediate_roll
                    < config.imm_zero_chance
                        + config.imm8s_min_chance
                        + config.imm8s_max_chance
                        + config.imm8s_inject_chance
                {
                    let index = rng.random_range(0..config.inject_values.len());
                    let inject_value = config.inject_values[index] as i8;
                    trace_log!(context, "Injecting immediate8s value {:02X}", inject_value);
                    iced_i.set_immediate8to16(inject_value as i16);
                    modified_iced = true;
                }
            }
            OpKind::Immediate16 => {
                // Roll for immediate override.
                let immediate_roll = rng.random_range(0.0..1.0);
                if immediate_roll < config.imm_zero_chance {
                    trace_log!(context, "Overriding immediate16 to zero");
                    iced_i.set_immediate16(0x0000);
                    modified_iced = true;
                }
                else if immediate_roll < config.imm_zero_chance + config.imm_ones_chance {
                    trace_log!(context, "Overriding immediate16 to all-ones");
                    iced_i.set_immediate16(0xFFFF);
                    modified_iced = true;
                }
                else if immediate_roll < config.imm_zero_chance + config.imm_ones_chance + config.imm_inject_chance {
                    let index = rng.random_range(0..config.inject_values.len());
                    let inject_value = config.inject_values[index] as u16;
                    trace_log!(context, "Injecting immediate16 value {:04X}", inject_value);
                    iced_i.set_immediate16(inject_value);
                    modified_iced = true;
                }
            }
            OpKind::Immediate32 => {
                // Roll for immediate override.
                let immediate_roll = rng.random_range(0.0..1.0);
                if immediate_roll < config.imm_zero_chance {
                    trace_log!(context, "Overriding immediate32 to zero");
                    iced_i.set_immediate32(0x0000_0000);
                    modified_iced = true;
                }
                else if immediate_roll < config.imm_zero_chance + config.imm_ones_chance {
                    trace_log!(context, "Overriding immediate32 to all-ones");
                    iced_i.set_immediate32(0xFFFF_FFFF);
                    modified_iced = true;
                }
                else if immediate_roll < config.imm_zero_chance + config.imm_ones_chance + config.imm_inject_chance {
                    let index = rng.random_range(0..config.inject_values.len());
                    let inject_value = config.inject_values[index];
                    trace_log!(context, "Injecting immediate32 value {:08X}", inject_value);
                    iced_i.set_immediate32(inject_value);
                    modified_iced = true;
                }
            }
            _ => {}
        }

        if modified_iced {
            let mut encoder = iced_x86::Encoder::new(context.code_segment_size.into());
            encoder.encode(&iced_i, 0)?;
            let buffer = encoder.take_buffer();

            // Iced will not encode multiple prefixes. If we generated multiple prefixes for this
            // instruction, it would be a pain to try to copy the iced-encoded instruction at the
            // correct spot, so instead we'll just replace the entire instruction bytes vector
            // with the new bytes. This means that we have a maximum of one segment override
            // prefix whenever we override an immediate, but this is an acceptable limitation.

            instruction_bytes = buffer.to_vec();
            decode_buffer = instruction_bytes.clone();
            decoder = Decoder::new(context.code_segment_size.into(), &decode_buffer, decoder_opts);
            iced_i = decoder.decode();
            instr_text = format_instruction(&iced_i);
            instruction_byte_ct = iced_i.len();
            sequence_bytes = instruction_byte_ct;

            let new_prefix_ct = count_prefixes(&instruction_bytes);
            let size_adjust = prefix_ct.saturating_sub(new_prefix_ct);
            if size_adjust > 0 {
                trace_log!(
                    context,
                    "Instruction shortened by {} bytes due to re-encoding",
                    size_adjust
                );
            }

            modrm_offset = modrm_offset.saturating_sub(size_adjust);

            trace_log!(
                context,
                "New instruction bytes: {:X?} ct:{} modrm_offset: {} size_adjust: {}",
                instruction_bytes,
                instruction_byte_ct,
                modrm_offset,
                size_adjust
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
                    "Invalid instruction length: {} for opcode {} (have {} instruction bytes)",
                    instruction_byte_ct,
                    opcode,
                    instruction_bytes.len()
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

        let base_segment = iced_i.memory_segment();
        log::debug!("Base segment is {:?}", base_segment);

        let operand_size = context
            .test_opcode_size_prefix
            .relative_opcode_size(context.code_segment_size);

        let address_size = context
            .test_opcode_size_prefix
            .relative_address_size(context.code_segment_size);

        let (displacement, d_offset) = get_displacement(&decoder, &iced_i, &instruction_bytes);
        let addressing_mode = calculate_addressing_mode(
            &instr_text,
            &instruction_bytes,
            base_segment,
            displacement,
            address_size,
            modrm_offset,
        );

        // Determine if instruction has modrm byte. There's no great way to get this info from iced that I can see...

        Ok(TestInstruction {
            name: instr_text,
            operand_size,
            address_size,
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
            addressing_mode,
            modrm_offset,
            displacement_offset: if d_offset > 0 { Some(d_offset) } else { None },
        })
    }

    pub fn mask_displacement32(
        &mut self,
        context: &mut TestContext,
        config: &TestGen,
        bitness: u32,
        mask: u32,
    ) -> anyhow::Result<()> {
        let mut modified_disp = false;

        let mut decoder_opts = DecoderOptions::NO_INVALID_CHECK;
        if matches!(config.cpu_type, MooCpuType::Intel80286) {
            decoder_opts |= DecoderOptions::LOADALL286;
        }

        if self.iced_i.memory_displ_size() == 4 {
            let mut disp32 = self.iced_i.memory_displacement32();
            let negative = disp32 & 0x8000_0000 != 0;

            disp32 &= mask;
            if negative {
                // Sign extend the masked displacement.
                disp32 |= !mask;
            }

            if disp32 != self.iced_i.memory_displacement32() {
                trace_log!(
                    context,
                    "Replacing Displacement32 value. Old: {:08X} New: {:08X}",
                    self.iced_i.memory_displacement32(),
                    disp32
                );
                self.iced_i.set_memory_displacement32(disp32);
                modified_disp = true;
            }
        }

        if modified_disp {
            let mut encoder = iced_x86::Encoder::new(bitness);
            encoder.encode(&self.iced_i, 0)?;
            let buffer = encoder.take_buffer();

            // Iced will not encode multiple prefixes. If we generated multiple prefixes for this
            // instruction, it would be a pain to try to copy the iced-encoded instruction at the
            // correct spot, so instead we'll just replace the entire instruction bytes vector
            // with the new bytes. This means that we have a maximum of one segment override
            // prefix whenever we override an immediate, but this is an acceptable limitation.

            trace_log!(
                context,
                "Old instruction bytes: {:02X?} ct:{}",
                self.bytes,
                self.iced_i.len()
            );

            let instruction_bytes = buffer.to_vec();
            self.bytes = instruction_bytes.clone();
            let mut decoder = Decoder::new(bitness, &instruction_bytes, decoder_opts);
            self.iced_i = decoder.decode();
            let instruction_byte_ct = self.iced_i.len();
            let mut sequence_bytes = instruction_byte_ct;

            let (displacement, d_offset) = get_displacement(&decoder, &self.iced_i, &instruction_bytes);

            let size_adjust = self.displacement_offset.unwrap_or(0).saturating_sub(d_offset);
            trace_log!(
                context,
                "Displacement offset adjusted from {:?} to {:?}, size adjust {}",
                self.displacement_offset,
                if d_offset > 0 { Some(d_offset) } else { None },
                size_adjust
            );
            self.modrm_offset = self.modrm_offset.saturating_sub(size_adjust);

            let base_segment = self.iced_i.memory_segment();
            let instr_text = format_instruction(&self.iced_i);
            self.addressing_mode = calculate_addressing_mode(
                &instr_text,
                &instruction_bytes,
                base_segment,
                displacement,
                self.address_size,
                self.modrm_offset,
            );

            if matches!(config.termination_condition, TerminationCondition::Halt) {
                // Insert a HALT instruction at the end of the sequence.
                if instruction_byte_ct == instruction_bytes.len() {
                    log::trace!("Appending HALT instruction");
                    // Decoded instruction uses all available bytes, so push a new HALT opcode.
                    sequence_bytes += 1;
                    self.bytes.push(0xF4); // HALT instruction for Intel 8086/8088.
                }
                else if instruction_byte_ct < instruction_bytes.len() {
                    log::trace!("Injecting HALT instruction at offset {}", instruction_byte_ct);
                    sequence_bytes += 1;
                    // Decoded bytes are less than instruction bytes, insert HALT opcode inline.
                    self.bytes[instruction_byte_ct] = 0xF4; // HALT instruction for Intel 8086/8088.
                }
                else {
                    // Bad condition
                    bail!(
                        "Invalid instruction length: {} for opcode {} (have {} instruction bytes)",
                        instruction_byte_ct,
                        self.opcode,
                        instruction_bytes.len()
                    );
                }
            }

            // Update the addressing mode.

            self.name = format_instruction(&self.iced_i);

            self.instr_range = Range {
                start: 0,
                end:   instruction_byte_ct,
            };
            self.sequence_range = Range {
                start: 0,
                end:   sequence_bytes,
            };

            trace_log!(
                context,
                "New instruction bytes: {:02X?} ct:{}",
                self.bytes,
                instruction_byte_ct
            );
        }

        Ok(())
    }

    pub fn mask_immediate32(
        &mut self,
        context: &mut TestContext,
        config: &TestGen,
        bitness: u32,
        mask: u32,
    ) -> anyhow::Result<()> {
        let mut modified_imm = false;
        let op1_kind = self.iced_i.op1_kind();

        let mut decoder_opts = DecoderOptions::NO_INVALID_CHECK;
        if matches!(config.cpu_type, MooCpuType::Intel80286) {
            decoder_opts |= DecoderOptions::LOADALL286;
        }

        match op1_kind {
            OpKind::Immediate32 => {
                log::debug!("Have Immediate32 to mask...");
                let mut imm32 = self.iced_i.immediate32();
                let negative = imm32 & 0x8000_0000 != 0;

                imm32 &= mask;
                if negative {
                    // Sign extend the masked immediate.
                    imm32 |= !mask;
                }

                if imm32 != self.iced_i.immediate32() {
                    log::debug!("Setting new Immediate32 value: {:08X}", imm32);
                    self.iced_i.set_immediate32(imm32);
                    modified_imm = true;
                }
            }
            _ => {
                return Ok(()); // Nothing to do
            }
        }

        if modified_imm {
            let mut encoder = iced_x86::Encoder::new(bitness);
            encoder.encode(&self.iced_i, 0)?;
            let buffer = encoder.take_buffer();

            // Iced will not encode multiple prefixes. If we generated multiple prefixes for this
            // instruction, it would be a pain to try to copy the iced-encoded instruction at the
            // correct spot, so instead we'll just replace the entire instruction bytes vector
            // with the new bytes. This means that we have a maximum of one segment override
            // prefix whenever we override an immediate, but this is an acceptable limitation.

            trace_log!(
                context,
                "Old instruction bytes: {:X?} ct:{}",
                self.bytes,
                self.iced_i.len()
            );

            let instruction_bytes = buffer.to_vec();
            self.bytes = instruction_bytes.clone();
            let mut decoder = Decoder::new(bitness, &instruction_bytes, decoder_opts);

            self.iced_i = decoder.decode();
            self.name = format_instruction(&self.iced_i);
            let instruction_byte_ct = self.iced_i.len();
            self.instr_range = Range {
                start: 0,
                end:   instruction_byte_ct,
            };
            self.sequence_range = Range {
                start: 0,
                end:   instruction_byte_ct,
            };

            trace_log!(
                context,
                "New instruction bytes: {:X?} ct:{}",
                self.bytes,
                instruction_byte_ct
            );
        }
        Ok(())
    }

    pub fn mask_nearbranch32(
        &mut self,
        context: &mut TestContext,
        config: &TestGen,
        bitness: u32,
        mask: u32,
    ) -> anyhow::Result<()> {
        let mut modified_imm = false;
        let op0_kind = self.iced_i.op0_kind();

        let mut decoder_opts = DecoderOptions::NO_INVALID_CHECK;
        if matches!(config.cpu_type, MooCpuType::Intel80286) {
            decoder_opts |= DecoderOptions::LOADALL286;
        }

        match op0_kind {
            OpKind::NearBranch32 => {
                log::trace!("Have NearBranch32 to mask...");
                let mut imm32 = self.iced_i.near_branch32();
                let negative = imm32 & 0x8000_0000 != 0;

                imm32 &= mask;
                if negative {
                    // Sign extend the masked immediate.
                    imm32 |= !mask;
                }

                if imm32 != self.iced_i.near_branch32() {
                    log::trace!("Setting new NearBranch32 value: {:08X}", imm32);
                    self.iced_i.set_near_branch32(imm32);
                    modified_imm = true;
                }
            }
            _ => {
                return Ok(()); // Nothing to do
            }
        }

        if modified_imm {
            let mut encoder = iced_x86::Encoder::new(bitness);
            encoder.encode(&self.iced_i, 0)?;
            let buffer = encoder.take_buffer();

            // Iced will not encode multiple prefixes. If we generated multiple prefixes for this
            // instruction, it would be a pain to try to copy the iced-encoded instruction at the
            // correct spot, so instead we'll just replace the entire instruction bytes vector
            // with the new bytes. This means that we have a maximum of one segment override
            // prefix whenever we override an immediate, but this is an acceptable limitation.

            trace_log!(
                context,
                "Old instruction bytes: {:02X?} ct:{}",
                self.bytes,
                self.iced_i.len()
            );

            let instruction_bytes = buffer.to_vec();
            self.bytes = instruction_bytes.clone();
            let mut decoder = Decoder::new(bitness, &instruction_bytes, decoder_opts);
            self.iced_i = decoder.decode();
            let instruction_byte_ct = self.iced_i.len();
            let mut sequence_bytes = instruction_byte_ct;

            if matches!(config.termination_condition, TerminationCondition::Halt) {
                // Insert a HALT instruction at the end of the sequence.
                if instruction_byte_ct == instruction_bytes.len() {
                    log::trace!("Appending HALT instruction");
                    // Decoded instruction uses all available bytes, so push a new HALT opcode.
                    sequence_bytes += 1;
                    self.bytes.push(0xF4); // HALT instruction for Intel 8086/8088.
                }
                else if instruction_byte_ct < instruction_bytes.len() {
                    log::trace!("Injecting HALT instruction at offset {}", instruction_byte_ct);
                    sequence_bytes += 1;
                    // Decoded bytes are less than instruction bytes, insert HALT opcode inline.
                    self.bytes[instruction_byte_ct] = 0xF4; // HALT instruction for Intel 8086/8088.
                }
                else {
                    // Bad condition
                    bail!(
                        "Invalid instruction length: {} for opcode {} (have {} instruction bytes)",
                        instruction_byte_ct,
                        self.opcode,
                        instruction_bytes.len()
                    );
                }
            }

            self.name = format_instruction(&self.iced_i);

            self.instr_range = Range {
                start: 0,
                end:   instruction_byte_ct,
            };
            self.sequence_range = Range {
                start: 0,
                end:   sequence_bytes,
            };

            trace_log!(
                context,
                "New instruction bytes: {:X?} ct:{}",
                self.bytes,
                instruction_byte_ct
            );
        }
        Ok(())
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

    pub fn ea_registers(&self) -> Vec<iced_x86::Register> {
        let mut info_factory = iced_x86::InstructionInfoFactory::new();
        let info = info_factory.info(&self.iced_i);

        let mut ea_regs = Vec::new();
        for m in info.used_memory() {
            if m.base() != iced_x86::Register::None && !ea_regs.contains(&m.base()) {
                ea_regs.push(m.base());
            }
            if m.index() != iced_x86::Register::None && !ea_regs.contains(&m.index()) {
                ea_regs.push(m.index());
            }
        }

        ea_regs
    }

    pub fn segments(&self) -> Vec<iced_x86::Register> {
        let mut info_factory = iced_x86::InstructionInfoFactory::new();
        let info = info_factory.info(&self.iced_i);

        let mut segments = Vec::new();
        for m in info.used_memory() {
            segments.push(m.segment());
        }

        segments
    }

    pub fn displacement_size(&self) -> Option<usize> {
        match self.iced_i.memory_displ_size() {
            0 => None,
            _ => Some(self.iced_i.memory_displ_size() as usize),
        }
    }

    pub fn immediate_size(&self) -> Option<usize> {
        match self.iced_i.op1_kind() {
            OpKind::Immediate8 | OpKind::Immediate8to16 | OpKind::Immediate8to32 => Some(1),
            OpKind::Immediate16 => Some(2),
            OpKind::Immediate32 | OpKind::Immediate32to64 => Some(4),
            OpKind::Immediate64 => Some(8),
            OpKind::Immediate8to64 => Some(8), // Always 8 bytes in
            _ => None,
        }
    }

    pub fn addressing_mode(&self) -> &Option<AddressingMode> {
        &self.addressing_mode
    }
}

pub fn get_displacement(
    decoder: &iced_x86::Decoder,
    instruction: &iced_x86::Instruction,
    bytes: &[u8],
) -> (Option<Displacement>, usize) {
    let constant_offsets = decoder.get_constant_offsets(instruction);
    let d_offset = constant_offsets.displacement_offset();
    match constant_offsets.displacement_size() {
        1 => {
            log::trace!("get_displacement(): Getting 8-bit displacement...");
            (Some(Displacement::Disp8(bytes[d_offset] as i8)), d_offset)
        }
        2 => {
            log::trace!("get_displacement(): Getting 16-bit displacement...");
            let disp_bytes = &bytes[d_offset..(d_offset + 2)];
            (
                Some(Displacement::Disp16(
                    u16::from_le_bytes(disp_bytes.try_into().unwrap()) as i16
                )),
                d_offset,
            )
        }
        4 => {
            let disp_bytes = &bytes[d_offset..(d_offset + 4)];
            let disp = u32::from_le_bytes(disp_bytes.try_into().unwrap());
            log::trace!(
                "get_displacement(): Getting 32-bit displacement from bytes {:X?} {:04X}",
                disp_bytes,
                disp
            );
            (
                Some(Displacement::Disp32(
                    u32::from_le_bytes(disp_bytes.try_into().unwrap()) as i32
                )),
                d_offset,
            )
        }
        _ => {
            log::trace!("get_displacement(): No displacement.");
            (None, 0)
        }
    }
}

pub fn calculate_addressing_mode(
    instr_text: &str,
    instruction_bytes: &[u8],
    base_segment: Register,
    displacement: Option<Displacement>,
    address_size: AddressSize,
    modrm_offset: usize,
) -> Option<AddressingMode> {
    // Gross hack to determine if we have modrm. Surely we can do better??
    let has_modrm = instr_text.contains("[");
    let displacement = displacement.unwrap_or(Displacement::NoDisp);
    if has_modrm {
        match address_size {
            AddressSize::Sixteen => {
                let modrm16 = ModRmByte16::read(instruction_bytes[modrm_offset]);

                if modrm16.is_addressing_mode() {
                    log::debug!(
                        "Have 16-bit addressing mode: {:?}:[{}]",
                        base_segment,
                        modrm16.address_offset(displacement)
                    );
                    return Some(AddressingMode::Sixteen(AddressingMode16::Address {
                        base:   base_segment.into(),
                        offset: modrm16.address_offset(displacement),
                    }));
                }
            }
            AddressSize::ThirtyTwo => {
                let modrm32 = ModRmByte32::read(instruction_bytes[modrm_offset]);

                return if modrm32.has_sib() {
                    let sib_byte = instruction_bytes[modrm_offset + 1];
                    let sib = SibByte::read(sib_byte, modrm32.mod_value());

                    log::debug!(
                        "Have 32-bit addressing mode with segment base {:?}, SIB byte {:02X} and displacement {}: {}",
                        base_segment,
                        sib_byte,
                        displacement,
                        sib.address_offset(displacement)
                    );
                    Some(AddressingMode::ThirtyTwo(AddressingMode32::Address {
                        base:   base_segment.into(),
                        offset: sib.address_offset(displacement),
                    }))
                }
                else {
                    if modrm32.is_addressing_mode() {
                        log::debug!(
                            "Have 32-bit addressing mode with segment base {:?}:  {}",
                            base_segment,
                            modrm32.address_offset(displacement)
                        );
                    }
                    Some(AddressingMode::ThirtyTwo(AddressingMode32::Address {
                        base:   base_segment.into(),
                        offset: modrm32.address_offset(displacement),
                    }))
                };
            }
        }
    }

    None
}
