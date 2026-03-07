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
use std::time::Instant;

use crate::{
    bus_ops::BusOps,
    cpu_common::{AddressingMode, AddressingMode16, AddressingMode32},
    cycles::MyServerCycleState,
    display::print_regs_v2,
    enums::{AddressSize, InstructionSize},
    generate::{
        gen_regs::TestRegisters,
        util::{
            adjust_flags_u16,
            adjust_flags_u32,
            calculate_ea,
            create_state,
            get_exception_hint,
            validate_disassembly,
        },
    },
    instruction::instruction::TestInstruction,
    registers::{compare_registers, validate_register_delta, Registers},
    state::{final_state_from_ops, initial_state_from_ops},
    trace_banner,
    trace_error,
    trace_log,
    trace_log::log_cycle_states,
    CpuMode,
    TestContext,
};

use anyhow::{anyhow, bail, Context, Error};
use arduinox86_client::{
    BinWrite,
    CpuWidth,
    MemoryStrategy,
    ProgramState,
    RegisterSetType,
    RemoteCpuRegistersV2,
    RemoteCpuRegistersV3B,
    ServerCpuType,
    ServerFlags,
};
use iced_x86::{Mnemonic, OpKind};
use marty_dasm::Opcode;
use moo::{
    prelude::*,
    registers::{MooRegisters16Printer, MooRegisters32Printer},
    types::{MooComparison, MooCpuType, MooStateType, MooTestGenMetadata},
};
use rand::{Rng, SeedableRng};

/// Generate a single test for the specified opcode and test number.
/// This will call `generate_test` repeatedly until a consistent result is obtained, the number of
/// times determined by the `validate_count` parameter in the test configuration.
///
/// `generate_consistent_test` is also responsible for sieving exceptions for opcodes configured
/// for exception sieving.
pub fn generate_consistent_test(
    ctx: &mut TestContext,
    test_num: usize,
    opcode: Opcode,
    opcode_ext: Option<u8>,
    required_matches: usize,
) -> Result<MooTest, Error> {
    let mut gen_num = 0;
    let mut sieved = false;
    let mut sieve_ct = 0;

    // Set flow control end condition
    if ctx.cfg.test_gen.flow_control_opcodes.contains(&opcode.into()) {
        let flags = ctx.client.get_flags()?;
        if flags & ServerFlags::HALT_AFTER_JUMP == 0 {
            // Enable halt after jump if not already set.
            ctx.client.set_flags(flags | ServerFlags::HALT_AFTER_JUMP)?;
            log::debug!("Enabled HALT_AFTER_JUMP for opcode {}", opcode);
        }
    }
    else {
        let flags = ctx.client.get_flags()?;
        if flags & ServerFlags::HALT_AFTER_JUMP != 0 {
            // Disable halt after jump if set.
            ctx.client.set_flags(flags & !ServerFlags::HALT_AFTER_JUMP)?;
            log::debug!("Disabled HALT_AFTER_JUMP for opcode {}", opcode);
        }
    }

    // We'll attempt to generate a test up to 'max_gen' times before giving up.
    // If we can't generate a test after that point, something has gone very wrong, like the
    // ArduinoX86 has crashed, the opcode is invalid, or we hit a major bug.
    while (sieved && sieve_ct < ctx.cfg.test_exec.max_sieve) || gen_num < ctx.cfg.test_exec.max_gen as usize {
        // Generate a fresh Register & Instruction pair.
        let mut test_registers = TestRegisters::new(ctx, opcode, test_num, gen_num);

        ctx.code_segment_size = test_registers.regs.segment_size(iced_x86::Register::CS);

        //log::trace!("Generating new instruction!");
        let mut test_instruction = TestInstruction::new(ctx, opcode, opcode_ext, &test_registers, test_num, gen_num)?;

        trace_banner!(ctx);
        trace_log!(ctx, "Code segment is size {:?}", ctx.code_segment_size);

        let mut segments = test_instruction.segments();
        let mut segment_limit = 0xFFFF_FFFF;

        for segment in &segments {
            trace_log!(
                ctx,
                "Test instruction accesses segment {:?} with limit {:X}, size {:?}",
                segment,
                test_registers.regs.segment_limit(*segment).unwrap_or(0),
                test_registers.regs.segment_size(*segment)
            );

            segment_limit &= test_registers.regs.segment_limit(*segment).unwrap_or(0xFFFF_FFFF);
        }

        let ea_registers = &test_instruction.ea_registers();
        for register in ea_registers {
            trace_log!(ctx, "Test instruction uses EA register {:?}", register);
        }

        let mut scale_shift = match test_instruction.iced_instruction().memory_index_scale() {
            8 => 3,
            4 => 2,
            2 => 1,
            _ => 0,
        };

        // If we have multiple EA registers, we need to scale the limit mask down further for every
        // additional register beyond the first.
        scale_shift += ea_registers.len().saturating_sub(1);

        if !ea_registers.is_empty() {
            trace_log!(
                ctx,
                "Masking EA registers: {:?} with limit {:08X} (shift: {})",
                ea_registers,
                segment_limit >> scale_shift,
                scale_shift
            );

            test_registers
                .regs
                .mask_registers32(segments[0], ea_registers, scale_shift as u32);
        }

        if ctx.cfg.test_gen.bitfield_opcodes.contains(&opcode.into()) {
            // Bitfield opcodes such as BT, BTS, BTR, BTC need the bit index masked if they use the
            // m32, reg form.
            if matches!(test_instruction.address_size(), AddressSize::ThirtyTwo) {
                if let OpKind::Register = test_instruction.op1_kind() {
                    if let OpKind::Memory = test_instruction.op0_kind() {
                        let register = test_instruction.iced_instruction().op1_register();
                        trace_log!(
                            ctx,
                            "Test instruction is bitfield operation with m, reg form. Masking register {:?}",
                            register
                        );

                        let reg_vec = vec![register];
                        test_registers.regs.mask_registers32(segments[0], &reg_vec, 0);
                    }
                }
            }
        }

        // Mask CX register if the instruction has REP/REPNE prefix.
        // ---------------------------------------------------------------------------------------------
        if test_instruction.iced_instruction().has_rep_prefix()
            || test_instruction.iced_instruction().has_repne_prefix()
        {
            // If the instruction has a REP or REPNE prefix, log it.
            trace_log!(
                ctx,
                "Instruction {} has REP/REPNE prefix. Masking CX with {:04X}",
                test_instruction.name(),
                ctx.cfg.test_gen.rep_cx_mask
            );

            match ctx.server_cpu {
                ServerCpuType::Intel80386 => {
                    let ecx = test_registers.regs.ecx();
                    test_registers.regs.set_ecx(ecx & ctx.cfg.test_gen.rep_cx_mask as u32);
                }
                _ => {
                    let cx = test_registers.regs.cx();
                    test_registers.regs.set_cx(cx & ctx.cfg.test_gen.rep_cx_mask);
                }
            }
        }

        // if ctx.cfg.test_gen.flow_control_opcodes.contains(&opcode.into()) {
        //     if matches!(test_instruction.iced_instruction().op0_kind(), OpKind::NearBranch32) {
        //         segment_limit &= test_registers
        //             .regs
        //             .segment_limit(iced_x86::Register::CS)
        //             .unwrap_or(0xFFFF_FFFF);
        //
        //         let segment_limit_shifted = segment_limit >> 1;
        //         trace_log!(ctx, "Test instruction is flow control operation with 32-bit near branch. Masking branch target with (segment limit >> 1): {:08X}", segment_limit_shifted);
        //
        //         test_instruction.mask_nearbranch32(ctx, &test_registers.regs, segment_limit_shifted)?;
        //     }
        // }

        if let Some(immediate_size) = test_instruction.immediate_size() {
            trace_log!(ctx, "Test instruction uses immediate of size {} bytes", immediate_size,);

            // if immediate_size > 2 &&
            //     trace_log!(
            //         ctx,
            //         "Instruction is a flow control operation with large immediate. Masking immediate with segment limit {:08X} (sign-extended)",
            //         segment_limit
            //     )
            // }
        }

        // Handle displacement masking.
        // 32-bit displacements are too large to fully randomize, so we need to mask them to the segment limit.
        if let Some(displacement_size) = test_instruction.displacement_size() {
            trace_log!(
                ctx,
                "Test instruction uses displacement of size {} bytes",
                displacement_size,
            );

            if displacement_size == 4 {
                let mut sign_extend = true;
                if ctx.cfg.test_gen.offset_opcodes.contains(&opcode.into()) {
                    // This is an instruction that uses an 32-bit offset parameter, like MOV to/from memory with a direct address.
                    // iced treats the offset as a displacement, but it should not be sign-extended like other masked offsets.

                    sign_extend = false;
                    log::trace!(
                        "Opcode {} uses direct offset: {:?}",
                        opcode,
                        test_instruction.iced_instruction()
                    );
                }

                // Handle POP.  POP will touch multiple segments, with the last segment being SS.
                // This isn't the segment we care about for masking so pop it.
                if matches!(test_instruction.iced_instruction().mnemonic(), Mnemonic::Pop) && segments.len() > 1 {
                    segments.pop();
                }

                // Handle 0xFF opcodes
                if matches!(
                    test_instruction.iced_instruction().mnemonic(),
                    Mnemonic::Call | Mnemonic::Jmp | Mnemonic::Push | Mnemonic::Pop
                ) && segments.len() > 1
                {
                    log::debug!("Have multiple segments, keeping first : {:?}", segments);
                    segments = vec![segments[0]];
                }

                if segments.len() > 1 {
                    let error_msg = format!(
                        "Multiple segments found with displacement: {:?} - unexpected condition.",
                        segments
                    );
                    trace_error!(ctx, "{}", error_msg);
                    bail!(error_msg);
                }
                else if !segments.is_empty() {
                    let segment = segments[0];
                    trace_log!(
                        ctx,
                        "Masking displacement with segment limit {:08X} (sign-extended)",
                        segment_limit
                    );

                    let scale_shift = match test_instruction.iced_instruction().memory_index_scale() {
                        8 => 3,
                        4 => 2,
                        2 => 1,
                        _ => 0,
                    };
                    test_instruction.mask_displacement32(
                        ctx,
                        &test_registers.regs,
                        sign_extend,
                        segment_limit >> scale_shift,
                    )?;
                }
                else if !matches!(test_instruction.iced_instruction().mnemonic(), Mnemonic::Lea) {
                    trace_error!(ctx, "No segment found with displacement and not LEA");
                    bail!("No segment found with displacement - unexpected condition.");
                }
            }
        }

        let mut test_attempt_ct = 0;
        let mut prev_test: Option<MooTest> = None;
        let mut match_count = 0;

        'gen: while test_attempt_ct < ctx.cfg.test_exec.test_retry {
            let test_result = generate_test(
                ctx,
                test_num,
                gen_num,
                opcode,
                opcode_ext,
                &test_instruction,
                &mut test_registers,
            );

            if ctx.dry_run {
                ctx.stats.add_mnemonic(test_instruction.mnemonic());
                return Err(anyhow!("Don't generate tests in dry run mode").into());
            }

            match test_result {
                Ok(test) => {
                    // Did test generate an exception?
                    if let Some(exception) = test.exception() {
                        // Handle exception sieving.
                        // To reduce the number of exceptions in exception-heavy instructions like BOUND,
                        // we reject tests that otherwise are good but generated an exception, at some
                        // predefined rate.
                        for es_entry in &ctx.cfg.test_gen.exception_sieve {
                            // If there is an opcode extension, check that the extension matches
                            // in the sieve entry. If no extension is given, match extension '0'.
                            let ext_matches = if let Some(opcode_ext) = opcode_ext {
                                es_entry.extension == opcode_ext
                            }
                            else {
                                es_entry.extension == 0
                            };

                            if es_entry.opcode == opcode.into()
                                && es_entry.exception == exception.exception_num
                                && ext_matches
                            {
                                log::debug!(
                                    "Opcode {} has sieve for exception {} at rate {}",
                                    opcode,
                                    exception.exception_num,
                                    es_entry.exception_rate
                                );

                                let mut rng = rand::rngs::StdRng::seed_from_u64(
                                    ctx.file_seed + test_num as u64 + sieve_ct as u64,
                                );
                                let roll: f32 = rng.random();
                                if roll < es_entry.exception_rate {
                                    log::warn!(
                                        "Sieve matched - accepting exception {} for opcode {}",
                                        exception.exception_num,
                                        opcode
                                    );
                                    trace_log!(
                                        ctx,
                                        "Sieve matched - accepting exception {} for opcode {}",
                                        exception.exception_num,
                                        opcode
                                    );
                                    sieved = false;
                                    sieve_ct = 0;
                                }
                                else {
                                    sieved = true;
                                    sieve_ct += 1;
                                    log::warn!(
                                        "Sieving exception {} for opcode {}, sieve_ct: {} gen_num: {}",
                                        exception.exception_num,
                                        opcode,
                                        sieve_ct,
                                        gen_num
                                    );
                                    trace_log!(
                                        ctx,
                                        "Sieve did not match (roll was {}) - rejecting exception {} for opcode {}, sieve_ct: {}",
                                        roll,
                                        exception.exception_num,
                                        opcode,
                                        sieve_ct
                                    );
                                    break 'gen;
                                }
                            }
                        }
                    }

                    if let Some(prev) = &prev_test {
                        let differences = prev.compare(&test, true);
                        let mut matched = false;

                        if differences.is_empty() {
                            matched = true;
                            match_count += 1;
                            if match_count >= required_matches - 1 {
                                trace_log!(
                                    ctx,
                                    "generate_consistent_test(): Test validation count met. Returning test."
                                );

                                // Our test is consistent, so we can update the test generation statistics now.
                                ctx.stats.total += 1;
                                ctx.stats.add_mnemonic(test_instruction.mnemonic());

                                let sib = test_instruction.has_sib();
                                if sib {
                                    ctx.stats.sib_ct += 1;
                                }

                                let modrm = test_instruction.has_modrm();
                                if !sib && modrm {
                                    ctx.stats.modrm_ct += 1;
                                }

                                if modrm {
                                    if test_instruction.has_modrm_register_mode() {
                                        ctx.stats.register_mode_ct += 1;
                                    }
                                    if test_instruction.has_modrm_address_mode() {
                                        ctx.stats.address_mode_ct += 1;
                                    }
                                }

                                if let Some(exception) = test.exception() {
                                    ctx.stats.add_exception(exception.exception_num, sib);
                                    if sib {
                                        ctx.stats.sib_exception_ct += 1;
                                    }
                                    else if modrm {
                                        ctx.stats.modrm_exception_ct += 1;
                                    }
                                }
                                else {
                                    if sib {
                                        ctx.stats.sib_no_exception_ct += 1;
                                    }
                                    else if modrm {
                                        ctx.stats.modrm_no_exception_ct += 1;
                                    }
                                }

                                return Ok(test);
                            }
                        }
                        else {
                            match differences.first().unwrap() {
                                MooComparison::RegisterMismatch => {
                                    trace_error!(
                                        ctx,
                                        "generate_consistent_test(): Register mismatch with previous test."
                                    );
                                    compare_registers(&test.final_state().regs(), prev.final_state().regs());
                                }
                                MooComparison::MemoryAddressMismatch(prev, current) => {
                                    trace_error!(
                                    ctx,
                                    "generate_consistent_test(): Memory address mismatch. Current: {:?} Previous: {:?}",
                                    current,
                                    prev
                                );
                                }
                                MooComparison::MemoryValueMismatch(prev, current) => {
                                    trace_error!(
                                    ctx,
                                    "generate_consistent_test(): Memory value mismatch. Current: {:?} Previous: {:?}",
                                    current,
                                    prev
                                );
                                }
                                MooComparison::CycleCountMismatch(prev, current) => {
                                    trace_error!(
                                        ctx,
                                        "generate_consistent_test(): Cycle count mismatch. Current: {} Previous: {}",
                                        current,
                                        prev
                                    );
                                }
                                MooComparison::CycleAddressMismatch(prev, current) => {
                                    trace_error!(
                                    ctx,
                                    "generate_consistent_test(): Cycle address mismatch. Current: {:06X} Previous: {:06X}",
                                    current,
                                    prev
                                );
                                }
                                MooComparison::CycleBusMismatch(prev, current) => {
                                    trace_error!(
                                    ctx,
                                    "generate_consistent_test(): Cycle bus mismatch. Current: {:1X} Previous: {:1X}",
                                    current,
                                    prev
                                );
                                }
                                MooComparison::ALEMismatch(cycle_n, prev, current) => {
                                    trace_error!(
                                    ctx,
                                    "generate_consistent_test(): ALE mismatch at cycle {}. Current: {} Previous: {}",
                                    cycle_n,
                                    current,
                                    prev
                                );
                                }
                                _ => {
                                    trace_error!(
                                        ctx,
                                        "generate_consistent_test(): Unknown mismatch with previous test: {:?}",
                                        differences
                                    );
                                }
                            }
                        }

                        if !matched {
                            trace_log!(ctx, "Test passed but did not match previous. Resetting match count.");
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
                        ctx,
                        "Failed to generate test for opcode {}, attempt {}: {}",
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
            ctx,
            "Retrying with new instruction generation (attempt {}/{})",
            gen_num,
            ctx.cfg.test_exec.max_gen
        );
    }

    let error_msg = format!(
        "Failed to generate consistent test for opcode {} after {} instruction generations",
        opcode, ctx.cfg.test_exec.max_gen
    );
    trace_error!(ctx, "{}", error_msg);
    Err(anyhow::anyhow!(error_msg).into())
}

pub fn generate_test(
    ctx: &mut TestContext,
    test_num: usize,
    gen_num: usize,
    opcode: Opcode,
    op_ext: Option<u8>,
    test_instruction: &TestInstruction,
    test_registers: &mut TestRegisters,
) -> anyhow::Result<MooTest> {
    let disassembly_failed = !validate_disassembly(ctx, opcode, test_instruction)?;

    // Log the start of instruction execution.
    // ---------------------------------------------------------------------------------------------
    test_instruction.log_instruction(ctx, test_num, opcode, op_ext, test_registers);

    if ctx.dry_run {
        bail!("Dry run mode enabled, skipping test generation.");
    }

    // Enable serial debug if configured.
    // ---------------------------------------------------------------------------------------------
    if Some(test_num) == ctx.cfg.test_exec.serial_debug_test {
        log::debug!("Enabling serial debug for test number {}", test_num);
        ctx.client.enable_debug(true)?;
    }
    else {
        ctx.client.enable_debug(ctx.cfg.test_exec.serial_debug_default)?;
    }

    // Generate test seed.
    // ---------------------------------------------------------------------------------------------
    let mut rng = rand::rngs::StdRng::seed_from_u64(ctx.file_seed);
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
    ctx.client.randomize_memory(test_seed as u32)?;

    // Determine the memory strategy based on the zero and ff chances.
    // ---------------------------------------------------------------------------------------------
    let strategy_chance: f32 = rng.random();
    let strategy = if strategy_chance < ctx.cfg.test_gen.mem_zero_chance {
        // Use zero memory strategy.
        trace_log!(ctx, "Using zero memory strategy");
        MemoryStrategy::Zero
    }
    else if strategy_chance < ctx.cfg.test_gen.mem_zero_chance + ctx.cfg.test_gen.mem_ones_chance {
        // Use ff memory strategy.
        trace_log!(ctx, "Using ff memory strategy");
        MemoryStrategy::Ones
    }
    else {
        // Use random memory strategy.
        trace_log!(ctx, "Using random memory strategy");
        MemoryStrategy::Random
    };

    // Set memory strategy on the client.
    ctx.client.set_memory_strategy(
        strategy,
        ctx.cfg.test_gen.mem_strategy_start,
        ctx.cfg.test_gen.mem_strategy_end,
    )?;

    // Calculate the effective address of the instruction, if any.
    let ea = calculate_ea(ctx, test_instruction, test_registers);

    if let Some(ea) = &ea {
        ea.trace_log(ctx);
    }

    // Patch memory if required by instruction type and cpu mode.
    // In real mode, we limit pointers to 16 bits.
    if matches!(strategy, MemoryStrategy::Random) {
        if matches!(
            ctx.test_opcode_size_prefix.relative_opcode_size(ctx.code_segment_size),
            InstructionSize::ThirtyTwo
        ) {
            // 32-bit instruction
            if test_instruction.is_near_indirect() | test_instruction.is_far_indirect() {
                // Instruction is an indirect call or jump. We will need to patch memory with a valid 32-bit pointer.

                let type_str = if test_instruction.is_near_indirect() {
                    "Near"
                }
                else {
                    "Far"
                };
                trace_log!(
                    ctx,
                    "{} indirect jump/call detected, addressing mode: {:X?}",
                    type_str,
                    test_instruction.addressing_mode()
                );

                match test_instruction.addressing_mode() {
                    Some(AddressingMode::Sixteen(AddressingMode16::Address { base, offset })) => {
                        if let CpuMode::Real = ctx.cfg.test_gen.cpu_mode {
                            trace_log!(
                                ctx,
                                "REAL MODE: Patching memory for 32-bit pointer opcode {}. Effective Segment: {:?} Base: {:08X}",
                                opcode,
                                base,
                                test_registers.regs.segment_base16(*base),
                            );

                            if let Registers::V3A(regs32) = &test_registers.regs {
                                let linear_address = offset.calculate_effective_address(*base, regs32);
                                log::warn!("16-bit effective address is {:08X}", linear_address,);

                                let mut mem_value = ctx.client.read_u32(linear_address)?;
                                trace_log!(
                                    ctx,
                                    "Patching value at 16-bit effective address {:08X}: {:08X}",
                                    linear_address,
                                    mem_value
                                );

                                mem_value &= 0x0000_FFFF;
                                ctx.client.write_u32(linear_address, mem_value)?;
                            }
                        }
                    }
                    Some(AddressingMode::ThirtyTwo(AddressingMode32::Address { base, offset })) => {
                        if let CpuMode::Real = ctx.cfg.test_gen.cpu_mode {
                            trace_log!(
                                ctx,
                                "REAL MODE: Patching memory for 32-bit pointer opcode {}. Effective Segment: {:?} Base: {:08X}",
                                opcode,
                                base,
                                test_registers.regs.segment_base32(*base),
                            );

                            trace_log!(ctx, "Patching memory for 32-bit pointer opcode {}", opcode);

                            if let Registers::V3A(regs32) = &test_registers.regs {
                                let linear_address = offset.calculate_effective_address(*base, regs32);
                                trace_log!(ctx, "32-bit effective address is {:08X}", linear_address);

                                let mut mem_value = ctx.client.read_u32(linear_address)?;
                                trace_log!(
                                    ctx,
                                    "Patching value at 32-bit effective address {:08X}: {:08X}",
                                    linear_address,
                                    mem_value
                                );

                                mem_value &= 0x0000_FFFF;

                                ctx.client.write_u32(linear_address, mem_value)?;
                            }
                        }
                    }
                    _ => {
                        let err_str = format!(
                            "Unsupported addressing mode for far indirect jump/call: {:?}",
                            test_instruction.addressing_mode()
                        );
                        trace_error!(ctx, "{}", err_str);
                        log::error!("{}", err_str);
                    }
                }
            }

            // Patch offsets for return instructions.
            if test_instruction.is_return() || test_instruction.is_iret() {
                let type_str = if test_instruction.is_near_return() {
                    "Near"
                }
                else if test_instruction.is_far_return() {
                    "Far"
                }
                else {
                    "Interrupt"
                };
                trace_log!(ctx, "{} return detected.", type_str,);

                if let CpuMode::Real = ctx.cfg.test_gen.cpu_mode {
                    trace_log!(
                        ctx,
                        "REAL MODE: Patching memory for 32-bit return opcode {}. CS Base: {:08X}",
                        opcode,
                        test_registers.regs.cs_base(),
                    );

                    let stack_address = test_registers.regs.stack_address();
                    trace_log!(ctx, "Stack address is {:08X}", stack_address);

                    let mem_value = ctx.client.read_u32(stack_address)?;
                    let new_value = mem_value & 0x0000_FFFF;

                    trace_log!(
                        ctx,
                        "Patching value at 32-bit stack address {:08X}: {:08X}->{:08X}",
                        stack_address,
                        mem_value,
                        new_value
                    );

                    ctx.client.write_u32(stack_address, new_value)?;
                }
            }
        }
    }

    // Upload the instruction sequence.
    log::trace!("Uploading instruction sequence...");
    ctx.client
        .set_memory(test_registers.instruction_address, test_instruction.sequence_bytes())?;

    let end_address = test_registers.instruction_address + test_instruction.sequence_bytes().len() as u32;
    ctx.client
        .set_program_bounds(test_registers.instruction_address, end_address)?;

    // Fix up memory if necessary.
    match test_instruction.operand_size() {
        InstructionSize::Sixteen => {
            adjust_flags_u16(ctx, test_seed, test_instruction, test_registers)?;
        }
        InstructionSize::ThirtyTwo => {
            adjust_flags_u32(ctx, test_seed, test_instruction, test_registers)?;
        }
    }

    //adjust_memory(ctx, test_seed, test_instruction, test_registers)?;

    if matches!(ctx.server_cpu, ServerCpuType::Intel80386) {
        // Set the jump hint flag if applicable.
        if let Some(odd) = test_instruction.branch_is_odd(ctx, &test_registers.regs) {
            match odd {
                true => {
                    trace_log!(ctx, "Detected odd branch. Setting jump hint flag to 1");
                    ctx.client.set_jump_hint(Some(true))?;
                }
                false => {
                    trace_log!(ctx, "Detected even branch. Setting jump hint flag to 0");
                    ctx.client.set_jump_hint(Some(false))?;
                }
            }
        }
        else {
            // Clear jump hint if no branch instruction detected.
            ctx.client.set_jump_hint(None)?;
        }
    }

    // Load the registers onto the Arduino.
    // ---------------------------------------------------------------------------------------------

    // Determine server program state. If we're in SMM mode we will need to convert to V3B registers.
    let state = ctx.client.get_program_state()?;

    // Reset cursor before writing to buffer!
    ctx.load_register_buffer.set_position(0);

    let load_type = match state {
        ProgramState::StoreDoneSmm => {
            log::trace!("Server in SMM. Converting registers to V3B for loading.");

            match &test_registers.regs {
                Registers::V3A(v3a_regs) => {
                    let v3b = RemoteCpuRegistersV3B::from(v3a_regs);
                    v3b.write_le(&mut ctx.load_register_buffer)?;

                    RegisterSetType::Intel386Smm
                }
                _ => {
                    unimplemented!(
                        "Unsupported register set type for SMM mode: {:?}",
                        ctx.register_set_type
                    );
                }
            }
        }
        _ => {
            test_registers.regs.to_buffer(&mut ctx.load_register_buffer);
            ctx.register_set_type
        }
    };

    let mut load_attempt_ct = 1;
    log::trace!(
        "Uploading registers, attempt {}/{}",
        load_attempt_ct,
        ctx.cfg.test_exec.load_retry
    );

    if let Err(e) = ctx
        .client
        .load_registers_from_buf(load_type, ctx.load_register_buffer.get_ref())
    {
        // If the load fails, retry up to `config.test_exec.load_retry` times.
        while load_attempt_ct < ctx.cfg.test_exec.load_retry {
            load_attempt_ct += 1;
            log::trace!(
                "Retrying register upload, attempt {}/{}",
                load_attempt_ct,
                ctx.cfg.test_exec.load_retry
            );
            if ctx
                .client
                .load_registers_from_buf(load_type, ctx.load_register_buffer.get_ref())
                .is_ok()
            {
                break;
            }
        }
        if load_attempt_ct >= ctx.cfg.test_exec.load_retry {
            bail!("Failed to upload registers after {} attempts: {}", load_attempt_ct, e);
        }
    }

    // Poll program state until finished with execution.
    // ---------------------------------------------------------------------------------------------
    let mut state = ctx.client.get_program_state()?;

    let start_time = Instant::now();
    while !matches!(
        state,
        ProgramState::StoreDone | ProgramState::StoreDoneSmm | ProgramState::Shutdown | ProgramState::Error
    ) {
        // Sleep for a little bit so we're not spamming the Arduino.
        std::thread::sleep(std::time::Duration::from_millis(ctx.cfg.test_exec.polling_sleep.into()));

        let millis = start_time.elapsed().as_millis() as u32;
        if millis > ctx.cfg.test_exec.test_timeout {
            let error_str = format!(
                "Test timeout reached after {} ms, program state is: {:?}",
                millis, state
            );
            trace_error!(ctx, "{}", error_str);
            bail!("{}", error_str);
        }
        state = ctx.client.get_program_state()?;
    }

    if matches!(state, ProgramState::Error) {
        log::error!("Error executing instruction: {}", ctx.client.get_last_error()?);

        ctx.last_program_state = Some(ProgramState::Error);
        return Err(anyhow::anyhow!(
            "Error executing instruction: {}",
            ctx.client.get_last_error()?
        ));
    }

    if matches!(state, ProgramState::Shutdown) {
        log::error!("Shutdown executing instruction: {}", ctx.client.get_last_error()?);

        ctx.last_program_state = Some(ProgramState::Shutdown);
        return Err(anyhow::anyhow!(
            "Shutdown executing instruction: {}",
            ctx.client.get_last_error()?
        ));
    }

    // Read the registers back from the Arduino.
    // ---------------------------------------------------------------------------------------------
    log::trace!("Reading registers back from ArduinoX86...");
    let reg_type = ctx
        .client
        .store_registers_to_buf(&mut ctx.store_register_buffer)
        .map_err(|e| anyhow::anyhow!("Error reading registers: {}", e))?;

    let final_regs = match reg_type {
        0x0 => {
            // V1 registers
            unimplemented!()
        }
        0x1 => {
            // V2 registers
            let regs_v2 = RemoteCpuRegistersV2::try_from(ctx.store_register_buffer.as_slice())
                .map_err(|e| anyhow::anyhow!("Error parsing V2 registers: {}", e))?;

            if ctx.cfg.test_exec.print_final_regs {
                print_regs_v2(&regs_v2, ctx.cfg.test_gen.cpu_type.into());
            }
            Registers::V2(regs_v2)
        }
        0x3 => {
            // V3B registers
            let regs_v3b = RemoteCpuRegistersV3B::try_from(ctx.store_register_buffer.as_slice())
                .map_err(|e| anyhow::anyhow!("Error parsing V3B registers: {}", e))?;

            if ctx.cfg.test_exec.print_final_regs {
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
    let cycle_states = ctx.client.get_cycle_states()?;
    log::trace!("Got {} cycle states!", cycle_states.len(),);

    let mut my_cycle_vec = Vec::new();

    // Convert cycle states to MooCycleStates.
    let mut moo_cycle_states = Vec::with_capacity(cycle_states.len());
    for cycle_state in &cycle_states {
        let my_cycle = match ctx.cfg.test_gen.cpu_type {
            MooCpuType::Intel80286 => MyServerCycleState::State286(cycle_state.clone()),
            MooCpuType::Intel80386Ex => MyServerCycleState::State386Ex(cycle_state.clone()),
            _ => unimplemented!(
                "Unsupported CPU type for cycle state conversion: {:?}",
                ctx.cfg.test_gen.cpu_type
            ),
        };
        my_cycle_vec.push(my_cycle.clone());
        moo_cycle_states.push(MooCycleState::from(my_cycle));
    }

    log_cycle_states(ctx, &moo_cycle_states);

    // Collect BusOps from cycle states.
    // ---------------------------------------------------------------------------------------------
    let mut bus_ops = BusOps::from(my_cycle_vec.as_slice());
    log::trace!("Got {} bus operations from cycles", bus_ops.len(),);

    bus_ops.detect_pushes(&test_registers.regs);
    bus_ops.log(ctx);

    if let Err(e) = final_regs.validate() {
        log::error!("Register validation failed: {}", e);
        trace_log!(ctx, "Register validation failed: {}", e);
        return Err(e);
    }

    if let Err(e) = validate_register_delta(
        test_instruction.iced_instruction().mnemonic(),
        &test_registers.regs,
        &final_regs,
    ) {
        log::error!("Register delta validation failed: {}", e);
        trace_log!(ctx, "Register delta validation failed: {}", e);
        return Err(e);
    }

    // Calculate initial memory state from bus operations.
    // ---------------------------------------------------------------------------------------------
    let initial_state = initial_state_from_ops(
        CpuWidth::from(ctx.server_cpu),
        test_registers.regs.cs_base(),
        test_registers.regs.ip(),
        test_instruction.sequence_bytes(),
        0,
        &bus_ops,
    )?;

    log::trace!("Got {} initial RAM entries", initial_state.initial_ram.len());

    // Get exception hint from cycle states.
    let exception_hint = get_exception_hint(ctx, &moo_cycle_states);

    // Detect any exceptions from bus operations.
    // ---------------------------------------------------------------------------------------------
    let operand_size = ctx.test_opcode_size_prefix.relative_opcode_size(ctx.code_segment_size);
    let exception = bus_ops.detect_exception(
        ctx,
        ctx.server_cpu.into(),
        operand_size,
        &test_registers.regs,
        &final_regs,
    )?;

    if let Err(e) = bus_ops.validate(
        ctx,
        &test_registers.regs,
        opcode,
        test_instruction.iced_instruction(),
        test_instruction.op0_kind(),
        test_instruction.op1_kind(),
        exception.is_some(),
    ) {
        log::error!("Bus operation validation failed: {}", e);
        trace_log!(ctx, "Bus operation validation failed: {}", e);
        return Err(e);
    }

    if let Some(exception) = &exception {
        if exception_hint.is_none() || !exception_hint.unwrap() {
            let err_str = format!(
                "Exception detected but no exception hint was set. Exception: {:?}",
                exception
            );
            log::warn!("{}", err_str);
            trace_error!(ctx, "{}", err_str);
        }

        log::trace!("Detected exception: {}", exception.exception_num);

        if !ctx.cfg.test_gen.allowed_exceptions.contains(&exception.exception_num) {
            let mut allowed = false;
            let op_ext_match = op_ext.unwrap_or(0);
            for entry in &ctx.cfg.test_gen.exception_overrides {
                if entry.opcode == opcode.into() && entry.extension == op_ext_match {
                    if entry.allow_all || entry.exceptions.contains(&exception.exception_num) {
                        trace_log!(
                            ctx,
                            "Exception {} allowed for opcode {}.{:X} via override.",
                            exception.exception_num,
                            opcode,
                            op_ext_match
                        );
                        allowed = true;
                        break;
                    }
                }
            }

            if !allowed {
                let err_str = format!("Exception {} not allowed by policy.", exception.exception_num);
                log::error!("{}", err_str);
                trace_error!(ctx, "{}", err_str);
                return Err(anyhow::anyhow!(err_str));
            }
        }

        trace_log!(ctx, "Detected exception: {}", exception.exception_num);
        trace_log!(ctx, "Flags on stack at {:06X}", exception.flag_address);
    }
    else {
        if let Some(true) = exception_hint {
            let err_str = String::from("No exception detected but exception hint was set.");
            log::warn!("{}", err_str);
            trace_error!(ctx, "{}", err_str);
        }
    }

    if disassembly_failed && (exception.is_none() || exception.as_ref().unwrap().exception_num != 6) {
        let err_str = format!(
            "Instruction disassembly failed and exception is not #UD (6). Exception: {:?}",
            exception
        );
        log::error!("{}", err_str);
        trace_error!(ctx, "{}", err_str);
        return Err(anyhow::anyhow!(err_str));
    }

    // Log final register state.
    // ---------------------------------------------------------------------------------------------
    match ctx.cfg.test_gen.cpu_type {
        MooCpuType::Intel80286 => {
            trace_log!(
                ctx,
                "{}",
                MooRegisters16Printer {
                    regs: &MooRegisters16::try_from(&final_regs)
                        .expect("Failed to convert final registers to MooRegisters16"),
                    cpu_type: ctx.cfg.test_gen.cpu_type,
                    diff: None,
                    indent: 0,
                }
            );
        }
        MooCpuType::Intel80386Ex => {
            trace_log!(
                ctx,
                "{}",
                MooRegisters32Printer {
                    regs: &MooRegisters32::try_from(&final_regs)
                        .expect("Failed to convert final registers to MooRegisters32"),
                    cpu_type: ctx.cfg.test_gen.cpu_type,
                    diff: None,
                    indent: 0,
                }
            );
        }
        _ => {}
    }

    // Calculate final memory state from initial state and bus operations.
    // ---------------------------------------------------------------------------------------------
    let final_ram = final_state_from_ops(initial_state.initial_state, &bus_ops)?;

    // Convert EA to MooEA
    let moo_ea = ea.as_ref().map(|ea| ea.into_moo());

    // Create the initial test state.
    let initial_state = create_state(
        MooStateType::Initial,
        &test_registers.regs,
        moo_ea,
        None,
        &initial_state.initial_ram,
    )?;
    // Create the final test state.
    let final_state = create_state(
        MooStateType::Final,
        &test_registers.regs,
        None,
        Some(&final_regs),
        &final_ram,
    )?;

    // Create the test case.
    let test = MooTest::new(
        test_instruction.name().into(),
        Some(gen_metadata),
        test_instruction.sequence_bytes(),
        initial_state,
        final_state,
        &moo_cycle_states,
        exception,
        None,
    );

    Ok(test)
}
