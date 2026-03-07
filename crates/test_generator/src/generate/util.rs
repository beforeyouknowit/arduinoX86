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
    cpu_common::{AddressingMode, AddressingMode16, AddressingMode32, EffectiveAddress, SegmentRegister},
    generate::gen_regs::TestRegisters,
    instruction::instruction::TestInstruction,
    registers::Registers,
    trace_error,
    trace_flush,
    trace_log,
    TestContext,
};

use arduinox86_client::ServerCpuType;
use marty_dasm::Opcode;
use moo::{
    prelude::*,
    types::{effective_address::MooEffectiveAddress, MooRamEntry, MooStateType, MooTestState},
};

use crate::enums::CpuMode;
use anyhow::bail;
use iced_x86::Mnemonic;
use rand::{Rng, SeedableRng};

pub fn validate_disassembly(
    ctx: &mut TestContext,
    opcode: Opcode,
    test_instruction: &TestInstruction,
) -> anyhow::Result<bool> {
    let name = test_instruction.name();

    let is_bad = name.contains("(bad)");

    if is_bad {
        return Ok(false);
    }

    if ctx.cfg.test_gen.offset_opcodes.contains(&opcode.into()) {
        // Will disassemble with brackets, but no modrm.
        return Ok(true);
    }

    if let (Some(start), Some(end)) = (name.find('['), name.find(']')) {
        if start < end {
            let iced_addr = &name[start..=end];

            if let Some(addressing_mode) = test_instruction.addressing_mode() {
                let marty_addr = addressing_mode.to_string();

                if iced_addr != marty_addr {
                    let err_str = format!("Disassembly mismatch! Iced: {}  Marty: {}", iced_addr, marty_addr);
                    log::error!("{}", err_str);
                    trace_error!(ctx, "{}", err_str);
                    trace_flush!(ctx);
                    std::process::exit(1);
                }
            }
            else {
                let err_str = format!(
                    "Instruction disassembly {} has addressing mode but no addressing mode was set",
                    name
                );
                trace_error!(ctx, "{}", err_str);
                log::error!("{}", err_str);
                bail!(err_str);
            }
        }
    }

    Ok(true)
}

pub fn calculate_ea(
    ctx: &mut TestContext,
    test_instruction: &TestInstruction,
    test_registers: &TestRegisters,
) -> Option<EffectiveAddress> {
    match test_instruction.addressing_mode() {
        Some(AddressingMode::Sixteen(AddressingMode16::Address { base, offset })) => {
            let segment_reg: SegmentRegister = base.try_into().expect("Failed to convert EA base to segment register");
            let iced_reg: iced_x86::Register = segment_reg.into();

            let mut ea = EffectiveAddress {
                base_segment: base
                    .try_into()
                    .expect("calculate_ea(): Couldn't convert 16-bit register to segment register"),
                base_segment_selector: test_registers
                    .regs
                    .segment_selector(iced_reg)
                    .expect("Failed to get segment selector for EA base register"),
                base_segment_address: test_registers
                    .regs
                    .segment_base(iced_reg)
                    .expect("Failed to get segment base for EA base register"),
                base_segment_limit: test_registers
                    .regs
                    .segment_limit(iced_reg)
                    .expect("Failed to get segment limit for EA base register"),
                ..Default::default()
            };

            match &test_registers.regs {
                Registers::V3A(regs32) => {
                    ea.offset = offset.calculate(regs32);
                    ea.linear_address = offset.calculate_effective_address(*base, regs32);
                    match ctx.cfg.test_gen.cpu_mode {
                        CpuMode::Unreal | CpuMode::Real => {
                            ea.physical_address = ea.linear_address;
                        }
                        _ => {}
                    }
                    Some(ea)
                }
                _ => {
                    unimplemented!(
                        "Unsupported register set type for EA calculation: {:?}",
                        ctx.register_set_type
                    );
                }
            }
        }
        Some(AddressingMode::ThirtyTwo(AddressingMode32::Address { base, offset })) => {
            let segment_reg: SegmentRegister = base.try_into().expect("Failed to convert EA base to segment register");
            let iced_reg: iced_x86::Register = segment_reg.into();
            let mut ea = EffectiveAddress {
                base_segment: base
                    .try_into()
                    .expect("Couldn't convert 32-bit segment register to segment register"),
                base_segment_selector: test_registers
                    .regs
                    .segment_selector(iced_reg)
                    .expect("Failed to get segment selector for EA base register"),
                base_segment_address: test_registers
                    .regs
                    .segment_base(iced_reg)
                    .expect("Failed to get segment base for EA base register"),
                base_segment_limit: test_registers
                    .regs
                    .segment_limit(iced_reg)
                    .expect("Failed to get segment limit for EA base register"),
                ..Default::default()
            };

            match &test_registers.regs {
                Registers::V3A(regs32) => {
                    ea.offset = offset.calculate(regs32);
                    ea.linear_address = offset.calculate_effective_address(*base, regs32);
                    match ctx.cfg.test_gen.cpu_mode {
                        CpuMode::Unreal | CpuMode::Real => {
                            ea.physical_address = ea.linear_address;
                        }
                        _ => {}
                    }
                    Some(ea)
                }
                _ => {
                    unimplemented!(
                        "Unsupported register set type for EA calculation: {:?}",
                        ctx.register_set_type
                    );
                }
            }
        }
        _ => None,
    }
}

/// Get a hint if an exception was expected from the cycle states.
pub fn get_exception_hint(ctx: &mut TestContext, cycles: &[MooCycleState]) -> Option<bool> {
    match ctx.server_cpu {
        ServerCpuType::Intel80386 => {
            // Check for ten consecutive INTA cycles.
            let mut inta_count = 0;
            for cycle in cycles {
                if cycle.bus_state == 0 {
                    inta_count += 1;
                    if inta_count >= 10 {
                        trace_log!(ctx, "Detected 10 consecutive INTA cycles, exception hint set.");
                        return Some(true);
                    }
                }
                else {
                    inta_count = 0;
                }
            }
            Some(false)
        }
        _ => None,
    }
}

pub fn adjust_flags_u16(
    ctx: &mut TestContext,
    test_seed: u64,
    test_instruction: &TestInstruction,
    test_registers: &mut TestRegisters,
) -> anyhow::Result<()> {
    // If the instruction is POPF, we need to generate a flag value without the trap flag.
    match test_instruction.iced_instruction().mnemonic() {
        Mnemonic::Popf => {
            // Generate a random flag value without the trap flag.
            let mut rng = rand::rngs::StdRng::seed_from_u64(test_seed);
            let flags = rng.random::<u16>() & (Registers::POP_FLAGS_MASK as u16);

            // Calculate the stack address.
            let stack_address = test_registers.regs.stack_address();
            trace_log!(
                ctx,
                "Writing new 16-bit POPF flags {:04X} to stack address {:#010X}",
                flags,
                stack_address
            );
            // Write the flags to the stack.
            ctx.client.set_memory(stack_address, &flags.to_le_bytes())?;
        }
        Mnemonic::Iret => {
            // Generate a random flag value without the trap flag.
            let mut rng = rand::rngs::StdRng::seed_from_u64(test_seed);
            let flags = rng.random::<u16>() & &(Registers::POP_FLAGS_MASK as u16);

            // Calculate the stack address. It's +4 because we need to write the flags, CS, and IP.
            let mut stack_address = test_registers.regs.ss_base();
            stack_address += test_registers.regs.sp().wrapping_add(4) as u32;
            trace_log!(
                ctx,
                "Writing new 16-bit IRET flags {:04X} to stack address {:#010X}",
                flags,
                stack_address
            );

            // Write the flags to the stack.
            ctx.client.set_memory(stack_address, &flags.to_le_bytes())?;
        }
        _ => {}
    }
    Ok(())
}

pub fn adjust_flags_u32(
    ctx: &mut TestContext,
    test_seed: u64,
    test_instruction: &TestInstruction,
    test_registers: &mut TestRegisters,
) -> anyhow::Result<()> {
    // If the instruction is POPF, we need to generate a flag value without the trap flag.
    match test_instruction.iced_instruction().mnemonic() {
        Mnemonic::Popfd => {
            // Generate a random flag value without the trap flag.
            let mut rng = rand::rngs::StdRng::seed_from_u64(test_seed);
            let flags = rng.random::<u32>() & Registers::POP_FLAGS_MASK;

            // Calculate the stack address.
            let stack_address = test_registers.regs.stack_address();
            trace_log!(
                ctx,
                "Writing new 32-bit POPF flags {:08X} to stack address {:08X}",
                flags,
                stack_address
            );
            // Write the flags to the stack.
            ctx.client.set_memory(stack_address, &flags.to_le_bytes())?;
        }
        Mnemonic::Iretd => {
            // Generate a random flag value without the trap flag.
            let mut rng = rand::rngs::StdRng::seed_from_u64(test_seed);
            let flags = rng.random::<u32>() & Registers::POP_FLAGS_MASK;

            // Calculate the stack address. It's +8 because we need to write the flags, CS, and IP.
            let mut stack_address = test_registers.regs.ss_base();
            stack_address += test_registers.regs.sp().wrapping_add(8) as u32;
            trace_log!(
                ctx,
                "Writing new 32-bit IRET flags {:08X} to stack address {:08X}",
                flags,
                stack_address
            );
            // Write the flags to the stack.
            ctx.client.set_memory(stack_address, &flags.to_le_bytes())?;
        }
        _ => {}
    }
    Ok(())
}

pub fn create_state(
    state_type: MooStateType,
    initial_regs: &Registers,
    ea: Option<MooEffectiveAddress>,
    final_regs: Option<&Registers>,
    ram: &Vec<[u32; 2]>,
) -> anyhow::Result<MooTestState> {
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

    let test_state = MooTestState::new(
        state_type,
        &initial_reg_init,
        final_reg_init.as_ref(),
        ea,
        Vec::new(),
        ram_vec,
    );

    if !test_state.regs().is_valid() {
        log::error!("Invalid registers in test state!");
        bail!("Invalid registers in test state");
    }

    Ok(test_state)
}
