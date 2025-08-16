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
use crate::{display::print_regs, registers::Registers, Config, CpuMode, TestContext, TestGen};
use arduinox86_client::{
    registers_common::RandomizeOpts,
    Registers32,
    RemoteCpuRegistersV1,
    RemoteCpuRegistersV2,
    RemoteCpuRegistersV3A,
    RemoteCpuRegistersV3B,
};
use moo::types::{MooCpuType, MooRegisters, MooRegisters16, MooRegisters32};
use rand::{rngs::StdRng, SeedableRng};
use rand_distr::Beta;
use std::ops::Range;

pub struct TestRegisters {
    pub regs: Registers,
    pub reg_seed: u64,
    pub instruction_address: u32,
}

impl From<&MooRegisters> for TestRegisters {
    fn from(regs: &MooRegisters) -> Self {
        match regs {
            MooRegisters::Sixteen(regs) => TestRegisters::from(regs),
            MooRegisters::ThirtyTwo(regs) => TestRegisters::from(regs),
        }
    }
}

impl From<&MooRegisters32> for TestRegisters {
    fn from(regs: &MooRegisters32) -> Self {
        let mut v3b = RemoteCpuRegistersV3B {
            eax: regs.eax,
            ebx: regs.ebx,
            ecx: regs.ecx,
            edx: regs.edx,
            esp: regs.esp,
            ebp: regs.ebp,
            esi: regs.esi,
            edi: regs.edi,
            cs: regs.cs as u16,
            ds: regs.ds as u16,
            es: regs.es as u16,
            fs: regs.fs as u16,
            gs: regs.gs as u16,
            ss: regs.ss as u16,
            eip: regs.eip,
            eflags: regs.eflags,
            ..Default::default()
        };
        v3b.normalize_descriptors();
        TestRegisters {
            regs: Registers::V3B(v3b),
            reg_seed: 0, // Seed not applicable for conversion
            instruction_address: (regs.cs << 4) + regs.eip,
        }
    }
}

impl From<&MooRegisters16> for TestRegisters {
    fn from(regs: &MooRegisters16) -> Self {
        let mut v2 = RemoteCpuRegistersV2 {
            ax: regs.ax,
            bx: regs.bx,
            cx: regs.cx,
            dx: regs.dx,
            sp: regs.sp,
            bp: regs.bp,
            si: regs.si,
            di: regs.di,
            cs: regs.cs,
            ds: regs.ds,
            es: regs.es,
            ss: regs.ss,
            ip: regs.ip,
            flags: regs.flags,
            ..Default::default()
        };
        v2.normalize_descriptors();
        TestRegisters {
            regs: Registers::V2(v2),
            reg_seed: 0, // Seed not applicable for conversion
            instruction_address: ((regs.cs as u32) << 4) + (regs.ip as u32),
        }
    }
}

impl TestRegisters {
    pub fn new(context: &mut TestContext, config: &Config, opcode: u8, test_num: usize, gen_number: usize) -> Self {
        // Put the gen_number into the top 8 bits of the test seed.
        // This allows us to generate tests based off the test number and gen count together.
        let reg_seed = context.file_seed ^ ((test_num as u64) | ((gen_number as u64) << 24) | 0x8000_0000);

        // Create a new rng seeded by the base seed XOR test seed for repeatability.
        let mut rng = StdRng::seed_from_u64(reg_seed);

        // Randomize the registers.
        let instruction_range: Range<u32> = Range {
            start: config.test_gen.instruction_address_range[0],
            end:   config.test_gen.instruction_address_range[1],
        };
        let mut registers_good = false;
        let mut instruction_address = 0;
        let mut initial_regs = Registers::V1(Default::default());

        // Repeatedly generate random registers until one set qualifies as valid.
        while !registers_good {
            initial_regs = match config.test_gen.cpu_type {
                MooCpuType::Intel80286 => {
                    let mut random_v2 = Registers::V2(RemoteCpuRegistersV2::default());
                    randomize_v2(context, config.test_gen.clone(), opcode, &mut rng, &mut random_v2);

                    if config.test_exec.print_initial_regs {
                        print_regs(&random_v2, config.test_gen.cpu_type.into());
                    }
                    random_v2
                }
                MooCpuType::Intel80386Ex => {
                    let mut random_v3a = Registers::V3A(RemoteCpuRegistersV3A::default());
                    randomize_v3a(context, config.test_gen.clone(), opcode, &mut rng, &mut random_v3a);
                    if config.test_exec.print_initial_regs {
                        print_regs(&random_v3a, config.test_gen.cpu_type.into());
                    }
                    random_v3a
                }
                _ => Registers::V1(RemoteCpuRegistersV1::default()),
            };

            if matches!(config.test_gen.cpu_mode, CpuMode::Real) {
                // Doing real mode test. Normalize the segment descriptors.
                initial_regs.normalize_descriptors();
            }

            // Check if the instruction is valid with the current registers.
            instruction_address = initial_regs.calculate_code_address() & config.test_gen.address_mask;
            if instruction_range.contains(&instruction_address) {
                registers_good = true;
            }
        }

        TestRegisters {
            regs: initial_regs,
            reg_seed,
            instruction_address,
        }
    }
}

pub fn randomize_v2(_context: &mut TestContext, config: TestGen, opcode: u8, rng: &mut StdRng, regs: &mut Registers) {
    let mut sp_min = config.sp_min_value;
    let mut sp_max = config.sp_max_value;

    for sp_override in &config.sp_overrides {
        if sp_override.opcode == opcode {
            sp_min = sp_override.min;
            sp_max = sp_override.max;
            break;
        }
    }

    let random_opts = RandomizeOpts {
        weight_zero: config.reg_zero_chance,
        weight_ones: config.reg_ones_chance,
        weight_sp_odd: config.sp_odd_chance,
        sp_min_value: sp_min,
        sp_max_value: sp_max,
        randomize_flags: true,
        clear_trap_flag: true,
        clear_interrupt_flag: true,
        randomize_general: true,
        randomize_ip: true,
        ip_mask: config.ip_mask,
        randomize_x: false,
        randomize_msw: false,
        randomize_tr: false,
        randomize_ldt: false,
        randomize_segment_descriptors: false,
        randomize_table_descriptors: false,
        ..Default::default()
    };

    let mut reg_beta = Beta::new(config.register_beta[0], config.register_beta[1])
        .expect("Couldn't create beta function for register randomization");

    regs.randomize(&random_opts, rng, &mut reg_beta);
}

pub fn randomize_v3a(_context: &mut TestContext, config: TestGen, opcode: u8, rng: &mut StdRng, regs: &mut Registers) {
    let mut sp_min = config.sp_min_value;
    let mut sp_max = config.sp_max_value;

    for sp_override in &config.sp_overrides {
        if sp_override.opcode == opcode {
            sp_min = sp_override.min;
            sp_max = sp_override.max;
            break;
        }
    }

    let random_opts = RandomizeOpts {
        weight_zero: config.reg_zero_chance,
        weight_ones: config.reg_ones_chance,
        weight_sp_odd: config.sp_odd_chance,
        sp_min_value: sp_min,
        sp_max_value: sp_max,
        sp_min_value32: sp_min as u32,
        sp_max_value32: sp_max as u32,
        randomize_flags: true,
        clear_trap_flag: true,
        clear_interrupt_flag: true,
        randomize_general: true,
        randomize_ip: true,
        ip_mask: config.ip_mask,
        eip_mask: config.ip_mask as u32,
        randomize_x: false,
        randomize_msw: false,
        randomize_tr: false,
        randomize_ldt: false,
        randomize_segment_descriptors: false,
        randomize_table_descriptors: false,
    };

    let mut reg_beta = Beta::new(config.register_beta[0], config.register_beta[1])
        .expect("Couldn't create beta function for register randomization");

    regs.randomize(&random_opts, rng, &mut reg_beta);
}
