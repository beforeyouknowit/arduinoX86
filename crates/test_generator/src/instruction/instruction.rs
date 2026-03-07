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

use crate::disassembly::prefixes::count_prefixes;
use std::{collections::VecDeque, ops::Range};

use crate::{
    cpu_common::{AddressingMode, AddressingMode16, AddressingMode32, Displacement},
    enums::{AddressSize, InstructionSize, TerminationCondition},
    generate::gen_regs::TestRegisters,
    modrm::{ModRmByte16, ModRmByte32, SibByte},
    trace_banner,
    trace_log,
    TestContext,
};

use anyhow::{bail, Context};
use arduinox86_client::registers_common::SegmentSize;
use iced_x86::{MemorySize, OpCodeOperandKind, OpCodeTableKind, OpKind, Register};
use marty_dasm::Opcode;
use moo::types::MooCpuType;
use rand::{
    prelude::{IndexedRandom, StdRng},
    Rng,
    SeedableRng,
};
use rand_distr::{Beta, Distribution};

use crate::{
    config::TestGen,
    disassembly::iced::IcedDisassembly,
    instruction::instruction_name::InstructionName,
    registers::Registers,
};
use marty_dasm::prelude::{
    CpuType as MdCpuType,
    Decoder as MdDecoder,
    DecoderOptions as MdDecoderOptions,
    Format,
    SegmentSize as MdSegmentSize,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DisassemblyProvider {
    #[default]
    IcedX86,
    MartyDasm,
}

#[derive(Default)]
pub struct TestInstruction {
    is_valid: bool,
    name: InstructionName,
    operand_size: InstructionSize,
    address_size: AddressSize,
    opcode: Opcode,
    opcode_ext: Option<u8>,
    bytes: Vec<u8>,
    test_seed: u64,
    instr_range: Range<usize>,
    sequence_range: Range<usize>,
    prefix_range: Range<usize>,
    disassembly_provider: DisassemblyProvider,
    iced: IcedDisassembly,
    mnemonic: String,
    op0_kind: OpKind,
    op1_kind: OpKind,
    addressing_mode: Option<AddressingMode>,
    modrm_offset: usize,
}

// Create a TestInstruction from a byte slice, such as the bytes chunk array - this allows us
// to create a TestInstruction from an existing test, ie, for validation.
// impl From<(InstructionSize, AddressSize, &[u8])> for TestInstruction {
//     fn from(data: (InstructionSize, AddressSize, &[u8])) -> Self {
//
//
//         let iced_i = Decoder::new(data.0.into(), data.2, DecoderOptions::NO_INVALID_CHECK).decode();
//         let instr_range = 0..iced_i.len();
//         let sequence_range = 0..data.2.len();
//         let prefix_range = 0..0; // TODO: Ignore prefixes for now.
//
//         let info = iced_i.op_code();
//         let low = info.op_code() as u16; // low opcode byte (xx)
//
//         let opcode = match info.table() {
//             OpCodeTableKind::T0F => 0x0F00 | low, // 2-byte opcode:  0F xx  -> 0x0Fxx
//             _ => low,
//         };
//
//         let (name, mnemonic) = format_iced_instruction(&iced_i);
//         let is_valid = iced_i.code() != iced_x86::Code::INVALID;
//
//         let iced = IcedDisassembly {
//             iced_i,
//             disp: None,
//             disp_offset: 0,
//         };
//
//         TestInstruction {
//             is_valid,
//             name,
//             operand_size: data.0,
//             address_size: data.1,
//             opcode: opcode.into(),
//             opcode_ext: None, // TODO: implement this
//             bytes: data.2.to_vec(),
//             test_seed: 0, // No seed for static instructions
//             instr_range,
//             sequence_range,
//             prefix_range,
//             disassembly_provider: DisassemblyProvider::IcedX86,
//             iced,
//             mnemonic,
//             op0_kind: iced_i.op0_kind(),
//             op1_kind: iced_i.op1_kind(),
//             addressing_mode: None,
//             modrm_offset: 0,
//             displacement_offset: None,
//         }
//     }
// }

pub fn decode_marty_instruction(ctx: &TestContext, bytes: &[u8]) -> anyhow::Result<marty_dasm::prelude::Instruction> {
    // TODO: implement FROM for converting these types
    let marty_decoder_opts = MdDecoderOptions {
        cpu: match ctx.server_cpu.into() {
            MooCpuType::Intel8086 | MooCpuType::Intel8088 => MdCpuType::Intel808x,
            MooCpuType::NecV30 => MdCpuType::NecVx0,
            MooCpuType::Intel80186 => MdCpuType::Intel8018x,
            MooCpuType::Intel80286 => MdCpuType::Intel80286,
            MooCpuType::Intel80386Ex => MdCpuType::Intel80386,
            _ => panic!("Unsupported CPU type {:?}", ctx.server_cpu),
        },
        segment_size: match ctx.code_segment_size {
            SegmentSize::Sixteen => MdSegmentSize::Segment16,
            SegmentSize::ThirtyTwo => MdSegmentSize::Segment32,
        },
    };

    // Decode the instruction with marty_dasm.
    let marty_decode_buffer = bytes.to_vec();
    let mut marty_decoder = MdDecoder::new(marty_decode_buffer.as_slice(), marty_decoder_opts);
    let marty_i = marty_decoder.decode_next()?;

    Ok(marty_i)
}

pub fn format_marty_instruction(marty_i: &marty_dasm::prelude::Instruction) -> (String, String) {
    let mut instr_string = String::new();
    let mut mnemonic_string = String::new();
    let mut format_options = marty_dasm::prelude::FormatOptions::default();

    marty_dasm::prelude::NasmFormatter.format_instruction(&marty_i, &format_options, &mut instr_string);
    format_options.mnemonic_only = true;
    marty_dasm::prelude::NasmFormatter.format_instruction(&marty_i, &format_options, &mut mnemonic_string);
    (instr_string, mnemonic_string)
}

pub fn get_effective_segment(iced_i: &iced_x86::Instruction) -> Option<Register> {
    match iced_i.memory_segment() {
        Register::DS | Register::ES | Register::FS | Register::GS | Register::SS | Register::CS => {
            Some(iced_i.memory_segment())
        }
        _ => None,
    }
}

pub fn apply_extension(modrm: u8, opcode_ext: Option<u8>) -> u8 {
    // Set the reg field of the modrm to the extension value.
    if let Some(ext) = opcode_ext {
        // Set the reg field of the modrm to the extension value.
        (modrm & 0b1100_0111) | ((ext & 0x07) << 3)
    }
    else {
        modrm
    }
}

pub fn has_sib(modrm: u8) -> bool {
    if modrm >> 6 != 0b11 && (modrm & 0x07) == 0b100 {
        true
    }
    else {
        false
    }
}

pub fn generate_modrm(
    rng: &mut StdRng,
    size: AddressSize,
    allow_reg_form: bool,
    sib_chance: Option<f32>,
    extension: Option<u8>,
    mask: u8,
) -> u8 {
    let mut modrm: u8 = rng.random();

    // Mod of 0b11 is a register form, so if this is not allowed, re-roll the modrm until its Mod is
    // not 0b11.
    if !allow_reg_form {
        while modrm & 0b1100_0000 == 0b1100_0000 {
            modrm = rng.random();
        }
    }

    let is_register_form = modrm & 0b1100_0000 == 0b1100_0000;

    let mut gen_sib = false;
    if !is_register_form && matches!(size, AddressSize::ThirtyTwo) {
        // Roll for SIB.
        if let Some(chance) = sib_chance {
            gen_sib = rng.random_bool(chance.into());
        }
    }

    if sib_chance.is_some() {
        if gen_sib {
            // Set RM to 0b100 to force a SIB byte.
            modrm = (modrm & 0b1111_1000) | 0b0000_0100;
        }
        else if !is_register_form {
            if modrm & 0b1100_0000 != 0b1100_0000 {
                // Re-roll RM until it is not 0b100 to avoid a SIB byte.
                let mut rm: u8 = rng.random();
                while (rm & 0b0000_0111) == 0b0000_0100 {
                    rm = rng.random();
                }
                modrm = (modrm & 0b1111_1000) | (rm & 0b0000_0111);
            }
        }
    }

    modrm = apply_extension(modrm, extension);
    modrm &= mask;
    modrm
}

// Helper macro to insert or prepend a prefix byte, updating the modrm offset if necessary.
macro_rules! insert_prefix {
    ($buf:ident, $byte:expr, $rng:ident, $label:expr, $prefix_ct:ident, $modrm_offset:expr) => {{
        if $prefix_ct > 0 {
            let insert_index = $rng.random_range(0..$prefix_ct);
            log::trace!("Inserting {} prefix at index {}", $label, insert_index);
            $buf.insert(insert_index, $byte);
        }
        else {
            log::trace!("Prepending {} prefix", $label);
            $buf.push_front($byte);
        }
        $modrm_offset += 1;
        $prefix_ct += 1;
    }};
}

pub fn modify_branch(
    ctx: &mut TestContext,
    rng: &mut StdRng,
    op0_kind: OpKind,
    iced_i: &mut iced_x86::Instruction,
    registers: &Registers,
) -> bool {
    let mut modified_iced = false;

    let config = &ctx.cfg.test_gen;
    match op0_kind {
        OpKind::NearBranch16 => {
            let mut branch_val = iced_i.near_branch16() as u32;
            let mut relative_branch_val_u32 = iced_i.near_branch16().wrapping_sub(registers.ip()) as u32;
            let mut relative_branch_val_i8 = iced_i.near_branch16().wrapping_sub(registers.ip()) as i8;
            trace_log!(
                ctx,
                "NearBranch16 value: {:04X} relative: {:04X} ({}) eip: {:08x}",
                branch_val,
                relative_branch_val_u32,
                relative_branch_val_i8,
                registers.eip()
            );

            // One particular near branch offset on the 286 is banned because it jumps in such
            // a way that it aligns with normal prefetch order, and so the jump is undetectable
            // by ArduinoX86.
            // near_branch_ban can be adjusted per-cpu in the configuration file as needed.
            if config.near_branch_ban.contains(&(relative_branch_val_i8 as i32)) {
                trace_log!(
                    ctx,
                    "NearBranch16 with banned relative value: {:04X} ({})",
                    relative_branch_val_u32,
                    relative_branch_val_i8,
                );
                while config.near_branch_ban.contains(&(relative_branch_val_i8 as i32)) {
                    relative_branch_val_i8 = rng.random::<i8>();
                    relative_branch_val_u32 = relative_branch_val_i8 as u32;
                    trace_log!(
                        ctx,
                        "Trying new relative branch value: {:04X} ({})",
                        relative_branch_val_u32,
                        relative_branch_val_i8
                    );

                    branch_val = registers
                        .eip()
                        .wrapping_add(iced_i.len() as u32)
                        .wrapping_add_signed(relative_branch_val_i8 as i32);
                }
                trace_log!(ctx, "Overriding NearBranch16 value to {:04X}", branch_val as u16);
                iced_i.set_near_branch16(branch_val as u16);
                modified_iced = true;
            }
        }
        OpKind::NearBranch32 => {
            let mut branch_val = iced_i.near_branch32();
            let mut relative_branch_val_u32 = iced_i.near_branch32().wrapping_sub(registers.eip());
            let mut relative_branch_val_i32 = iced_i.near_branch32().wrapping_sub(registers.eip()) as i32;
            let mut relative_branch_val_i8 = relative_branch_val_i32 as i8;
            let cs_limit = registers.segment_limit(Register::CS).unwrap_or(0xFFFF);
            trace_log!(
                ctx,
                "NearBranch32 value: {:08X} relative: {} ({}) eip: {:08x} cs limit: {:04X}",
                branch_val,
                relative_branch_val_u32,
                relative_branch_val_i8,
                registers.eip(),
                cs_limit
            );

            if (branch_val > cs_limit) || config.near_branch_ban.contains(&relative_branch_val_i32) {
                trace_log!(
                    ctx,
                    "NearBranch32 with banned relative value: {:04X} ({})",
                    relative_branch_val_u32,
                    relative_branch_val_i32,
                );
                while (branch_val > cs_limit) || config.near_branch_ban.contains(&relative_branch_val_i32) {
                    relative_branch_val_i8 = rng.random::<i8>();
                    relative_branch_val_i32 = relative_branch_val_i8 as i32;
                    relative_branch_val_u32 = relative_branch_val_i8 as u32;
                    //relative_branch_val_i32 = rng.random_range((-((cs_limit / 2) as i32))..=((cs_limit / 2) as i32));
                    //relative_branch_val = relative_branch_val_i32 as u32;
                    trace_log!(
                        ctx,
                        "Trying new relative branch value: {} ({})",
                        relative_branch_val_u32,
                        relative_branch_val_i32
                    );

                    branch_val = registers
                        .eip()
                        .wrapping_add(iced_i.len() as u32)
                        .wrapping_add_signed(relative_branch_val_i32);
                }
                trace_log!(ctx, "Overriding base NearBranch32 value to {:04X}", branch_val);
                iced_i.set_near_branch32(branch_val);
                modified_iced = true;
            }
        }
        OpKind::FarBranch32 => {
            // 32-bit branches are problematic as their large magnitude will generate a
            // protection fault the vast majority of the time.

            // In real mode, we can simply mask the branch value to 16-bits as the segment limit
            // is known. In unreal or protected mode, we can't reliably predict the segment
            // limit.
            let mut branch_val = iced_i.far_branch32();
            trace_log!(ctx, "Far branch value: {:08X}", branch_val);

            // Mask the branch value to 16-bits as we can't predict the destination segment size...
            branch_val &= 0x0000_FFFF;
            trace_log!(ctx, "Masked Far branch value to {:08X}", branch_val);
            iced_i.set_far_branch32(branch_val);
            modified_iced = true;
        }
        _ => {}
    }

    modified_iced
}

pub fn modify_immediate_op0(
    ctx: &mut TestContext,
    rng: &mut StdRng,
    opcode: Opcode,
    op0_kind: OpKind,
    iced_i: &mut iced_x86::Instruction,
) -> bool {
    let mut modified_iced = false;

    match op0_kind {
        OpKind::Immediate8 => {
            let mut port = iced_i.immediate8();
            if ctx.cfg.test_gen.io_opcodes.contains(&opcode.into()) {
                // Check immediate value for IO port ban.
                while ctx.cfg.test_gen.io_port_ban.contains(&(port as u16)) {
                    let old_port = port;
                    port = rng.random();
                    trace_log!(
                        ctx,
                        "IO port {:02X} is banned. Generating new IO port immediate8 value: {:02X}",
                        old_port,
                        port
                    );
                }
                iced_i.set_immediate8(port);
                modified_iced = true;
            }
        }
        _ => {}
    }
    modified_iced
}

/// Helper function for [TestInstruction::new()] to modify immediate operands.
pub fn modify_immediate_op1(
    ctx: &mut TestContext,
    rng: &mut StdRng,
    opcode: Opcode,
    op1_kind: OpKind,
    iced_i: &mut iced_x86::Instruction,
) -> bool {
    let mut modified_iced = false;

    let config = &ctx.cfg.test_gen;
    match op1_kind {
        OpKind::Immediate8 => {
            if config.io_opcodes.contains(&opcode.into()) {
                // Check immediate value for IO port ban.
                let mut port = iced_i.immediate8();
                while config.io_port_ban.contains(&(port as u16)) {
                    let old_port = port;
                    port = rng.random();
                    trace_log!(
                        ctx,
                        "IO port {:02X} is banned. Generating new IO port immediate8 value: {:02X}",
                        old_port,
                        port
                    );
                }
                iced_i.set_immediate8(port);
                return true;
            }

            // iced considers rcl reg, 1 as an immediate8, and it is an error to override it
            // so only override the immediate if it is not 1.
            if iced_i.immediate8() != 0x01 {
                // Roll for immediate override.
                let immediate_roll = rng.random_range(0.0..1.0);
                if immediate_roll < config.imm_zero_chance {
                    trace_log!(ctx, "Overriding immediate8 to zero");
                    iced_i.set_immediate8(0x00);
                    modified_iced = true;
                }
                else if immediate_roll < config.imm_zero_chance + config.imm_ones_chance {
                    trace_log!(ctx, "Overriding immediate8 to all-ones");
                    iced_i.set_immediate8(0xFF);
                    modified_iced = true;
                }
                else if immediate_roll < config.imm_inject_chance {
                    let index = ctx.weighted_index.sample(rng);
                    let inject_value = config.inject_values[index].value as u8;
                    trace_log!(ctx, "Injecting immediate8 value {:02X}", inject_value);
                    iced_i.set_immediate8(inject_value);
                    modified_iced = true;
                }
            }
        }
        OpKind::Immediate8to16 => {
            // Roll for immediate override.
            let immediate_roll = rng.random_range(0.0..1.0);
            if immediate_roll < config.imm_zero_chance {
                trace_log!(ctx, "Overriding immediate8s to zero");
                iced_i.set_immediate8to16(0x0000);
                modified_iced = true;
            }
            else if immediate_roll < config.imm_zero_chance + config.imm8s_min_chance {
                trace_log!(ctx, "Overriding immediate8s to minimum");
                iced_i.set_immediate8to16(i16::MIN);
                modified_iced = true;
            }
            else if immediate_roll < config.imm_zero_chance + config.imm8s_min_chance + config.imm8s_max_chance {
                trace_log!(ctx, "Overriding immediate8s to maximum");
                iced_i.set_immediate8to16(i16::MAX);
                modified_iced = true;
            }
            else if immediate_roll
                < config.imm_zero_chance
                    + config.imm8s_min_chance
                    + config.imm8s_max_chance
                    + config.imm8s_inject_chance
            {
                let index = ctx.weighted_index.sample(rng);
                let inject_value = config.inject_values[index].value as i8;
                trace_log!(ctx, "Injecting immediate8s value {:02X}", inject_value);
                iced_i.set_immediate8to16(inject_value as i16);
                modified_iced = true;
            }
        }
        OpKind::Immediate16 => {
            // Roll for immediate override.
            let immediate_roll = rng.random_range(0.0..1.0);
            if immediate_roll < config.imm_zero_chance {
                trace_log!(ctx, "Overriding immediate16 to zero");
                iced_i.set_immediate16(0x0000);
                modified_iced = true;
            }
            else if immediate_roll < config.imm_zero_chance + config.imm_ones_chance {
                trace_log!(ctx, "Overriding immediate16 to all-ones");
                iced_i.set_immediate16(0xFFFF);
                modified_iced = true;
            }
            else if immediate_roll < config.imm_zero_chance + config.imm_ones_chance + config.imm_inject_chance {
                let index = ctx.weighted_index.sample(rng);
                let inject_value = config.inject_values[index].value as u16;
                trace_log!(ctx, "Injecting immediate16 value {:04X}", inject_value);
                iced_i.set_immediate16(inject_value);
                modified_iced = true;
            }
        }
        OpKind::Immediate32 => {
            // Roll for immediate override.
            let immediate_roll = rng.random_range(0.0..1.0);
            if immediate_roll < config.imm_zero_chance {
                trace_log!(ctx, "Overriding immediate32 to zero");
                iced_i.set_immediate32(0x0000_0000);
                modified_iced = true;
            }
            else if immediate_roll < config.imm_zero_chance + config.imm_ones_chance {
                trace_log!(ctx, "Overriding immediate32 to all-ones");
                iced_i.set_immediate32(0xFFFF_FFFF);
                modified_iced = true;
            }
            else if immediate_roll < config.imm_zero_chance + config.imm_ones_chance + config.imm_inject_chance {
                let index = ctx.weighted_index.sample(rng);
                let inject_value = config.inject_values[index].value;
                trace_log!(ctx, "Injecting immediate32 value {:08X}", inject_value);
                iced_i.set_immediate32(inject_value);
                modified_iced = true;
            }
        }
        _ => {}
    }

    modified_iced
}

impl TestInstruction {
    /// Generate a new, random [TestInstruction] for the specified [OpCode] given the provided
    /// [TestContext] and [TestGen] config.
    pub fn new(
        ctx: &mut TestContext,
        opcode: Opcode,
        opcode_ext: Option<u8>,
        test_registers: &TestRegisters,
        test_num: usize,
        gen_number: usize,
    ) -> anyhow::Result<Self> {
        // Check opcode is valid.
        if ctx.cfg.test_gen.excluded_opcodes.contains(&opcode.into()) {
            bail!("Opcode {} is excluded from generation", opcode);
        }
        if !opcode.is_extended() && ctx.cfg.test_gen.prefixes.contains(&opcode.into()) {
            bail!("Opcode {} is a prefix and cannot be generated", opcode);
        }

        trace_banner!(ctx);
        trace_log!(ctx, "Generating instruction for opcode {}", opcode);

        let mut inst = TestInstruction {
            opcode,
            // Put the gen_number into the top 8 bits of the test seed.
            // This allows us to generate tests based off the test number and gen count together.
            test_seed: ctx.file_seed ^ ((test_num as u64) | ((gen_number as u64) << 24)),
            // Set the operand and address sizes.
            // These are relative to the code segment size, which was set in the ctx.
            address_size: ctx.test_opcode_size_prefix.relative_address_size(ctx.code_segment_size),
            operand_size: ctx.test_opcode_size_prefix.relative_opcode_size(ctx.code_segment_size),
            ..TestInstruction::default()
        };

        // Create a new rng seeded by the base seed XOR test seed for repeatability.
        let mut rng = StdRng::seed_from_u64(inst.test_seed);

        // Create a deque to hold our instruction bytes and put the opcode byte(s) into it.
        let mut instruction_bytes: VecDeque<u8> = VecDeque::new();
        instruction_bytes.extend(opcode.to_bytes());

        // Now we want to generate a random modrm.
        // We can always generate a modrm even if the instruction doesn't need one - it will just
        // be interpreted as displacement/immediate byte or ignored if it is not needed.

        // First, calculate whether we should allow register forms or apply a modrm mask
        // (default mask is 0xFF to do nothing).

        let mut allow_reg_form = true;
        let mut modrm_mask = 0xFF;

        // The configuration file specifies 'modrm overrides' that can disallow register forms or
        // apply an arbitrary mask to the modrm byte during generation.
        // We will honor these overrides if the opcode and extension match - with the caveat that
        // the 'invalid_chance' parameter can cause the overrides to be ignored. This allows us to
        // sprinkle a few invalid forms in to the test file to ensure we have coverage of invalid
        // forms without allowing them to be too common.
        for mod_override in &ctx.cfg.test_gen.modrm_overrides {
            if (mod_override.opcode == opcode.into()) && (mod_override.extension == opcode_ext.unwrap_or(0)) {
                // Apply the specified modrm mask unless 'invalid_chance' is rolled.
                let allow_invalid = rng.random_bool(mod_override.invalid_chance.into());

                if allow_invalid {
                    trace_log!(
                        ctx,
                        "Allowing invalid modrm forms due to invalid_chance {:.2}",
                        mod_override.invalid_chance
                    );
                }

                if !allow_invalid && !mod_override.allow_reg_form {
                    trace_log!(ctx, "Disallowing register modrm forms due to modrm override.");
                    allow_reg_form = false;
                }

                if !allow_invalid {
                    trace_log!(ctx, "Applying modrm override mask of {:02X}", mod_override.mask);
                    modrm_mask = mod_override.mask;
                }
            }
        }

        // We will repeatedly generate a modrm until one is accepted by per-CPU filters.
        // Most of this per-CPU logic has been moved into the configuration file modrm override
        // section, but it is retained here if ever needed.
        let mut modrm_accepted = false;
        let mut modrm = 0x00;

        while !modrm_accepted {
            // Generate a modrm byte.
            modrm = generate_modrm(
                &mut rng,
                inst.address_size,
                allow_reg_form,
                ctx.cfg.test_gen.sib_chance,
                opcode_ext,
                modrm_mask,
            );

            // Check if the generated modrm is acceptable for the current CPU type.
            match ctx.cfg.test_gen.cpu_type {
                MooCpuType::Intel80286 | MooCpuType::Intel80386Ex => {
                    // Any modrm is fine for the 80286 as invalid forms will generate a UD exception
                    // instead of glitching out.
                    modrm_accepted = true;
                }
                _ => {
                    unimplemented!("Opcode generation for CPU type {:?}", ctx.cfg.test_gen.cpu_type);
                }
            }
        }

        // Push the modrm byte to the instruction deque and calculate its index.
        // We will adjust this index as we add prefixes.
        // We need the modrm index to be able to calculate effective addresses for memory operands
        // that need an override.
        instruction_bytes.push_back(modrm);
        inst.modrm_offset = instruction_bytes.len() - 1;
        trace_log!(
            ctx,
            "Instruction bytes: {:X?} Added modrm at offset {}",
            instruction_bytes,
            inst.modrm_offset
        );

        // Add 'instruction_pad' random bytes to the deque - this covers any displacement/immediate bytes.
        for _ in 0..ctx.cfg.test_gen.instruction_pad {
            let byte = rng.random();
            instruction_bytes.push_back(byte);
        }

        // Append specified opcode size prefixes, adjusting the modrm index as we go.
        for byte in Vec::<u8>::from(ctx.test_opcode_size_prefix) {
            instruction_bytes.push_front(byte);
            inst.modrm_offset += 1;
        }

        // If the configuration file specifies this CPU has segment override prefixes, we want
        // to randomly add some.

        // Start with 0 prefixes.
        {
            let config = &ctx.cfg.test_gen;
            let mut prefix_ct = 0;
            if !config.segment_prefixes.is_empty() {
                // If segment override prefixes are not disabled for this opcode, generate some.
                if !config.disable_seg_overrides.contains(&opcode.into()) {
                    // Create a beta distribution to determine the number of prefixes.
                    // This weights longer sequences lower, so the tests aren't dominated by long prefix
                    // sequences.
                    let reg_beta =
                        Beta::new(config.prefix_beta[0], config.prefix_beta[1]).expect("Invalid beta parameters");
                    let beta_out = reg_beta.sample(&mut rng);
                    prefix_ct = (beta_out * config.max_prefixes as f64).round() as usize;
                };

                // Add segment override prefixes, again adjusting the modrm index as bytes are pushed.
                for _i in 0..prefix_ct {
                    // Should be safe to unwrap() since we checked for empty segment_prefixes above.
                    let segment_prefix = config.segment_prefixes.choose(&mut rng).unwrap();
                    instruction_bytes.push_front(*segment_prefix);
                    inst.modrm_offset += 1;
                }
            }

            // Roll for LOCK prefix chance if the opcode isn't excluded from LOCK prefix generation.
            if !config.disable_lock_prefix.contains(&opcode.into()) {
                let have_lock_prefix = rng.random_bool(config.lock_prefix_chance.into());
                if have_lock_prefix {
                    insert_prefix!(
                        instruction_bytes,
                        config.lock_prefix_opcode,
                        rng,
                        "LOCK",
                        prefix_ct,
                        inst.modrm_offset
                    );
                }
            }

            // Roll for REP prefix chance if the opcode is on the list of opcodes that can have a REP prefix.
            if config.rep_opcodes.contains(&opcode.into()) {
                let have_rep_prefix = rng.random_bool(config.rep_prefix_chance.into());
                if have_rep_prefix {
                    if let Some(prefix) = config.rep_prefixes.choose(&mut rng) {
                        insert_prefix!(instruction_bytes, *prefix, rng, "REP", prefix_ct, inst.modrm_offset);
                    }
                }
            }
        }

        // Convert our Deque into a Vec.
        inst.bytes = instruction_bytes.into();
        trace_log!(ctx, "Final instruction bytes: {:X?}", inst.bytes);

        // Decode the instruction with iced-x86.
        inst.iced = IcedDisassembly::disassemble(ctx, &inst.bytes, &test_registers.regs);
        let is_iced_valid = inst.iced.is_valid();

        if !is_iced_valid {
            trace_log!(ctx, "Instruction decoded as INVALID by iced-x86");
        }

        let marty_i = decode_marty_instruction(ctx, &inst.bytes)?;
        let is_marty_valid = marty_i.is_valid;

        if !is_marty_valid {
            trace_log!(ctx, "Instruction decoded as INVALID by marty_dasm");
        }

        if !is_iced_valid || !is_marty_valid {
            trace_log!(ctx, "Using marty_dasm disassembly.");
            inst.disassembly_provider = DisassemblyProvider::MartyDasm;
        }

        let instruction_byte_ct = match inst.disassembly_provider {
            DisassemblyProvider::IcedX86 => {
                let (instr_text, mnemonic) = inst.iced.format();
                inst.name = InstructionName::from_iced(&instr_text);
                inst.mnemonic = mnemonic;

                let instruction_byte_ct = std::cmp::min(inst.iced.len(), inst.bytes.len());
                inst.instr_range.end = instruction_byte_ct;
                inst.sequence_range.end = instruction_byte_ct;
                instruction_byte_ct
            }
            DisassemblyProvider::MartyDasm => {
                let (iced_instr_text, _) = inst.iced.format();
                let (instr_text, mnemonic) = format_marty_instruction(&marty_i);
                inst.name = InstructionName::from_marty(&instr_text);
                inst.name.set_iced(&iced_instr_text);
                inst.mnemonic = mnemonic;

                let instruction_byte_ct = std::cmp::min(marty_i.instruction_bytes.len(), inst.bytes.len());
                inst.instr_range.end = instruction_byte_ct;
                inst.sequence_range.end = instruction_byte_ct;
                instruction_byte_ct
            }
        };

        let prefix_ct = count_prefixes(ctx, &inst.bytes);
        inst.prefix_range.end = prefix_ct;

        inst.handle_termination(ctx, instruction_byte_ct)?;

        // Get the effective segment for memory operands.
        let base_segment = inst.iced.i().memory_segment();
        log::trace!("Base segment is {:?}", base_segment);

        // Calculate the optional addressing mode for memory operands.
        // We use the addressing mode to patch memory operands.
        inst.addressing_mode = calculate_addressing_mode(
            ctx,
            &inst.name,
            &inst.bytes,
            opcode,
            base_segment,
            inst.iced.disp(),
            inst.address_size,
            inst.modrm_offset,
        );

        // Get the iced operand kinds from the decoded instruction. We use these to implement
        // instruction-specific overrides.
        inst.op0_kind = inst.iced.i().op0_kind();
        inst.op1_kind = inst.iced.i().op1_kind();

        // We can now modify the iced instruction in different ways, overriding operands and
        // re-encoding if necessary.

        // This flag controls whether we need to re-encode the instruction with iced.
        let mut modified_iced = false;

        // Override problematic near branch values.
        modified_iced |= modify_branch(
            ctx,
            &mut rng,
            inst.op0_kind,
            &mut inst.iced.i_mut(),
            &test_registers.regs,
        );

        // Override immediate operands.
        // We manipulate immediate operands to inject edge-case values.
        // Originally, the generator only supported overriding to 0 or all-ones, but we have since
        // added support for selecting values from an 'inject_values' array in the configuration.
        // This is preferred, but overriding to 0 or all-ones is still supported for backwards
        // compatibility.
        modified_iced |= modify_immediate_op0(ctx, &mut rng, opcode, inst.op0_kind, &mut inst.iced.i_mut());
        modified_iced |= modify_immediate_op1(ctx, &mut rng, opcode, inst.op1_kind, &mut inst.iced.i_mut());

        // If we modified the iced instruction, we need to re-encode it to get the new bytes.
        if modified_iced {
            inst.re_encode(ctx, &test_registers.regs)?;
        }

        Ok(inst)
    }

    pub fn load(
        ctx: &mut TestContext,
        opcode: Opcode,
        name: &str,
        bytes: &[u8],
        registers: &Registers,
    ) -> anyhow::Result<Self> {
        let mut inst = TestInstruction {
            opcode,
            name: InstructionName::from_resolved(name),
            bytes: bytes.to_vec(),
            // Set the operand and address sizes.
            // These are relative to the code segment size, which was set in the ctx.
            address_size: ctx.test_opcode_size_prefix.relative_address_size(ctx.code_segment_size),
            operand_size: ctx.test_opcode_size_prefix.relative_opcode_size(ctx.code_segment_size),
            ..TestInstruction::default()
        };

        inst.iced = IcedDisassembly::disassemble(ctx, &inst.bytes, registers);
        inst.instr_range = 0..inst.iced.len();
        inst.sequence_range = 0..bytes.len();
        inst.prefix_range = 0..count_prefixes(ctx, &bytes);

        let info = inst.iced.i().op_code();
        let low = info.op_code() as u16; // low opcode byte (xx)
        let decoded_opcode = match info.table() {
            OpCodeTableKind::Normal => low,
            OpCodeTableKind::T0F => 0x0F00 | low, // 2-byte opcode:  0F xx  -> 0x0Fxx
            _ => {
                log::warn!("Warning: Unsupported opcode: {:4X} table kind {:?}", low, info.table());
                low
            }
        };

        let (decoded_name, mnemonic) = inst.iced.format();
        inst.is_valid = inst.iced.is_valid();

        if inst.is_valid && (inst.name.resolved() != decoded_name) {
            log::warn!(
                "Warning: Instruction name '{}' does not match decoded name '{}'",
                name,
                decoded_name
            );
        }

        let raw_opcode: u16 = opcode.into();
        if inst.is_valid && (decoded_opcode != opcode.into()) {
            let mut opcode_matches = false;
            //println!("opkind: {:?}", info.op0_kind());

            // Iced annoyingly returns the base opcode for /r encoded opcodes, so we need to mask
            // here
            if matches!(
                info.op0_kind(),
                OpCodeOperandKind::r8_opcode | OpCodeOperandKind::r16_opcode | OpCodeOperandKind::r32_opcode
            ) {
                if (decoded_opcode & !0x07) == (raw_opcode & !0x07) {
                    opcode_matches = true;
                }
            }

            // Iced can also decode some funky two-byte opcodes when we have an invalid instruction form.
            if decoded_opcode > 0x0FFF {
                if ((decoded_opcode >> 8) & !0xFF) == (raw_opcode & !0xFF) {
                    opcode_matches = true;
                }
            }

            if !opcode_matches {
                bail!(
                    "Decoded instruction opcode {:X} does not match metadata opcode {}. OpCodeTableKind: {:?} Bytes: {:X?} Name: {}",
                    decoded_opcode,
                    opcode,
                    info.table(),
                    bytes,
                    name
                );
            }
        }

        inst.mnemonic = mnemonic;
        inst.bytes = bytes.to_vec();

        Ok(inst)
    }

    pub fn handle_termination(&mut self, ctx: &mut TestContext, new_byte_ct: usize) -> anyhow::Result<()> {
        if matches!(ctx.cfg.test_gen.termination_condition, TerminationCondition::Halt) {
            // Insert a HALT instruction at the end of the sequence.
            if new_byte_ct == self.bytes.len() {
                log::trace!("Appending HALT instruction");
                // Decoded instruction uses all available bytes, so push a new HALT opcode.
                self.sequence_range.end += 1;
                self.bytes.push(0xF4); // HALT instruction for Intel 8086/8088.
            }
            else if new_byte_ct < self.bytes.len() {
                log::trace!("Injecting HALT instruction at offset {}", new_byte_ct);
                self.sequence_range.end += 1;
                // Decoded bytes are less than instruction bytes, insert HALT opcode inline.
                self.bytes[new_byte_ct] = 0xF4; // HALT instruction for Intel 8086/8088.
            }
            else {
                // Bad condition
                bail!(
                    "Invalid instruction length: {} for opcode {} (have {} instruction bytes)",
                    new_byte_ct,
                    self.opcode,
                    self.bytes.len()
                );
            }
        }
        Ok(())
    }

    /// Re-encode the instruction with iced-x86, falling back to marty_dasm if invalid.
    pub fn re_encode(&mut self, ctx: &mut TestContext, registers: &Registers) -> anyhow::Result<()> {
        let mut encoder = iced_x86::Encoder::new(ctx.code_segment_size.into());

        match encoder.encode(&self.iced.i(), registers.eip() as u64) {
            Ok(_) => {}
            Err(e) => {
                bail!("Error re-encoding instruction with iced-x86: {}", e);
            }
        }
        let buffer = encoder.take_buffer();

        // Iced will not encode multiple prefixes. If we generated multiple prefixes for this
        // instruction, it would be a pain to try to copy the iced-encoded instruction at the
        // correct spot, so instead we'll just replace the entire instruction bytes vector
        // with the new bytes. This means that we have a maximum of one segment override
        // prefix whenever we override an immediate, but this is an acceptable limitation.

        trace_log!(
            ctx,
            "   > Re-encoding instruction. Old instruction bytes: {:02X?} ct:{}",
            self.bytes,
            self.iced.len()
        );

        let new_instruction_bytes = buffer.to_vec();

        let new_iced = IcedDisassembly::disassemble(ctx, &new_instruction_bytes, registers);
        let (new_instr_text, _new_mnemonic) = new_iced.format();
        let new_instruction_byte_ct = new_iced.len();
        let new_sequence_bytes = new_instruction_byte_ct;

        let old_prefix_ct = count_prefixes(ctx, &self.bytes);
        let new_prefix_ct = count_prefixes(ctx, &new_instruction_bytes);
        let size_adjust = old_prefix_ct.saturating_sub(new_prefix_ct);
        if size_adjust > 0 {
            trace_log!(
                ctx,
                "   > Instruction shortened by {} bytes due to re-encoding",
                size_adjust
            );
        }

        let new_modrm_offset = self.modrm_offset.saturating_sub(size_adjust);
        trace_log!(
            ctx,
            "   > New instruction bytes: {:X?} ct:{} modrm_offset: {} size_adjust: {}",
            new_instruction_bytes,
            new_instruction_byte_ct,
            new_modrm_offset,
            size_adjust
        );

        self.modrm_offset = new_modrm_offset;
        self.iced = new_iced;
        self.name = InstructionName::from_iced(&new_instr_text);
        self.bytes = new_instruction_bytes;
        self.instr_range.end = new_instruction_byte_ct;
        self.sequence_range.end = new_sequence_bytes;
        self.prefix_range.end = new_prefix_ct;

        // Update the addressing mode.
        let base_segment = self.iced.i().memory_segment();
        self.addressing_mode = calculate_addressing_mode(
            ctx,
            &self.name,
            &self.bytes,
            self.opcode,
            base_segment,
            self.iced.disp(),
            self.address_size,
            self.modrm_offset,
        );

        if let Some(addressing_mode) = &self.addressing_mode {
            trace_log!(ctx, "   > New calculated addressing mode: {}", addressing_mode);
        }
        else {
            trace_log!(ctx, "   > No addressing mode calculated");
        }

        self.handle_termination(ctx, new_instruction_byte_ct)?;

        trace_log!(
            ctx,
            "   > New instruction bytes: {:02X?} ct:{}",
            self.bytes,
            new_instruction_byte_ct
        );

        Ok(())
    }

    /// Mask a 32-bit displacement with the specified mask, optionally sign-extending the result.
    /// If the displacement is modified, the instruction is re-encoded with iced-x86 and
    /// the instruction bytes and disassembly text are updated.
    pub fn mask_displacement32(
        &mut self,
        ctx: &mut TestContext,
        registers: &Registers,
        sign_extend: bool,
        mask: u32,
    ) -> anyhow::Result<()> {
        let mut modified_disp = false;

        if self.iced.i().memory_displ_size() == 4 {
            let mut disp32 = self.iced.i().memory_displacement32();
            let negative = disp32 & 0x8000_0000 != 0;

            disp32 &= mask;
            if sign_extend && negative {
                // Sign extend the masked displacement.
                disp32 |= !mask;
            }

            if disp32 != self.iced.i().memory_displacement32() {
                trace_log!(
                    ctx,
                    "   > Replacing Displacement32 value. Old: {:08X} New: {:08X}",
                    self.iced.i().memory_displacement32(),
                    disp32
                );
                self.iced.i_mut().set_memory_displacement32(disp32);
                modified_disp = true;
            }
        }

        if modified_disp {
            self.re_encode(ctx, registers)?;
        }

        Ok(())
    }

    pub fn mask_immediate32(&mut self, ctx: &mut TestContext, registers: &Registers, mask: u32) -> anyhow::Result<()> {
        let mut modified_imm = false;
        let op1_kind = self.iced.i().op1_kind();

        match op1_kind {
            OpKind::Immediate32 => {
                log::debug!("Have Immediate32 to mask...");
                let mut imm32 = self.iced.i().immediate32();
                let negative = imm32 & 0x8000_0000 != 0;

                imm32 &= mask;
                if negative {
                    // Sign extend the masked immediate.
                    imm32 |= !mask;
                }

                if imm32 != self.iced.i().immediate32() {
                    log::debug!("Setting new Immediate32 value: {:08X}", imm32);
                    self.iced.i_mut().set_immediate32(imm32);
                    modified_imm = true;
                }
            }
            _ => {
                return Ok(()); // Nothing to do
            }
        }

        if modified_imm {
            self.re_encode(ctx, registers)?;
        }
        Ok(())
    }

    pub fn mask_nearbranch32(&mut self, ctx: &mut TestContext, registers: &Registers, mask: u32) -> anyhow::Result<()> {
        let mut modified_imm = false;
        let op0_kind = self.iced.i().op0_kind();

        match op0_kind {
            OpKind::NearBranch32 => {
                log::trace!("Have NearBranch32 to mask...");
                let mut imm32 = self.iced.i().near_branch32();
                let negative = imm32 & 0x8000_0000 != 0;

                imm32 &= mask;
                if negative {
                    // Sign extend the masked immediate.
                    imm32 |= !mask;
                }

                if imm32 != self.iced.i().near_branch32() {
                    log::trace!("Setting new NearBranch32 value: {:08X}", imm32);
                    self.iced.i_mut().set_near_branch32(imm32);
                    modified_imm = true;
                }
            }
            _ => {
                return Ok(()); // Nothing to do
            }
        }

        if modified_imm {
            self.re_encode(ctx, registers)?;
        }
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name.resolved()
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
        &self.iced.i()
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
        let info = info_factory.info(&self.iced.i());

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
        let info = info_factory.info(&self.iced.i());

        let mut segments = Vec::new();
        for m in info.used_memory() {
            segments.push(m.segment());
        }

        segments
    }

    pub fn displacement_size(&self) -> Option<usize> {
        match self.iced.i().memory_displ_size() {
            0 => None,
            _ => Some(self.iced.i().memory_displ_size() as usize),
        }
    }

    pub fn immediate_size(&self) -> Option<usize> {
        match self.iced.i().op1_kind() {
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

    pub fn is_near_indirect(&self) -> bool {
        match self.op0_kind() {
            // Indirect near jump/call (0xFF)
            OpKind::Memory => matches!(
                self.iced.i().memory_size(),
                MemorySize::WordOffset | MemorySize::DwordOffset
            ),
            _ => false,
        }
    }

    pub fn is_far_indirect(&self) -> bool {
        match self.op0_kind() {
            // Indirect far jump/call (0xFF)
            OpKind::Memory => matches!(self.iced.i().memory_size(), MemorySize::SegPtr16 | MemorySize::SegPtr32),
            _ => false,
        }
    }

    pub fn is_far_return(&self) -> bool {
        use iced_x86::Code;
        matches!(
            self.iced.i().code(),
            Code::Retfw | Code::Retfd | Code::Retfw_imm16 | Code::Retfd_imm16
        ) // CB, CA iw
    }

    pub fn is_near_return(&self) -> bool {
        use iced_x86::Code;
        matches!(
            self.iced.i().code(),
            Code::Retnw | Code::Retnd | Code::Retnw_imm16 | Code::Retnd_imm16
        ) // C3, C2 iw
    }

    pub fn is_return(&self) -> bool {
        self.is_near_return() || self.is_far_return()
    }

    pub fn is_iret(&self) -> bool {
        use iced_x86::Code;
        matches!(self.iced.i().code(), Code::Iretw | Code::Iretd) // CF, C7, C9
    }

    pub fn has_sib(&self) -> bool {
        if let Some(AddressingMode::ThirtyTwo(addr32)) = &self.addressing_mode {
            addr32.is_sib()
        }
        else {
            false
        }
    }

    pub fn has_modrm(&self) -> bool {
        if let Some(_) = &self.addressing_mode {
            true
        }
        else {
            false
        }
    }

    pub fn has_modrm_register_mode(&self) -> bool {
        if let Some(AddressingMode::Sixteen(addr16)) = &self.addressing_mode {
            addr16.is_register_mode()
        }
        else if let Some(AddressingMode::ThirtyTwo(addr32)) = &self.addressing_mode {
            addr32.is_register_mode()
        }
        else {
            false
        }
    }

    pub fn has_modrm_address_mode(&self) -> bool {
        if let Some(AddressingMode::Sixteen(addr16)) = &self.addressing_mode {
            addr16.is_address_mode()
        }
        else if let Some(AddressingMode::ThirtyTwo(addr32)) = &self.addressing_mode {
            addr32.is_address_mode()
        }
        else {
            false
        }
    }

    pub fn address_size(&self) -> AddressSize {
        self.address_size
    }

    pub fn operand_size(&self) -> InstructionSize {
        self.operand_size
    }

    pub fn opcode(&self) -> Opcode {
        self.opcode
    }

    pub fn branch_is_odd(&self, ctx: &mut TestContext, registers: &Registers) -> Option<bool> {
        match self.op0_kind() {
            OpKind::NearBranch16 => {
                let branch_val = self.iced.i().near_branch16();
                Some(branch_val & 1 != 0)
            }
            OpKind::NearBranch32 => {
                let branch_val = self.iced.i().near_branch32();
                Some(branch_val & 1 != 0)
            }
            _ => None,
        }
    }
}

pub fn has_modrm(ctx: &TestContext, instr_text: &str, opcode: Opcode) -> bool {
    // Gross hack to determine if we have modrm. Surely we can do better??
    let mut has_modrm = instr_text.contains("[");

    // Some instructions have modrm but don't use memory operands.
    if ctx.cfg.test_gen.offset_opcodes.contains(&opcode.into()) {
        has_modrm = false;
    }

    has_modrm
}

pub fn calculate_addressing_mode(
    ctx: &TestContext,
    instr_name: &InstructionName,
    instruction_bytes: &[u8],
    opcode: Opcode,
    base_segment: Register,
    displacement: Option<Displacement>,
    address_size: AddressSize,
    modrm_offset: usize,
) -> Option<AddressingMode> {
    let has_modrm = has_modrm(ctx, instr_name.iced(), opcode);
    let displacement = displacement.unwrap_or(Displacement::NoDisp);
    if has_modrm {
        match address_size {
            AddressSize::Sixteen => {
                let modrm16 = ModRmByte16::read(instruction_bytes[modrm_offset]);

                if modrm16.is_addressing_mode() {
                    log::trace!(
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

                    log::trace!(
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
                        log::trace!(
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
