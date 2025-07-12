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
use crate::display::print_regs;
use crate::{registers::Registers, Config, CpuMode, TestContext, TestGen};
use ard808x_client::{RandomizeOpts, RemoteCpuRegistersV1, RemoteCpuRegistersV2};
use moo::types::MooCpuType;
use rand::rngs::StdRng;
use rand_distr::Beta;
use std::ops::Range;

pub struct TestRegisters {
    pub regs: Registers,
    pub instruction_address: u32,
}

impl TestRegisters {
    pub fn new(context: &mut TestContext, config: &Config, rng: &mut StdRng) -> Self {
        // Randomize the registers.
        let instruction_range: Range<u32> = Range {
            start: config.test_gen.instruction_address_range[0],
            end: config.test_gen.instruction_address_range[1],
        };
        let mut registers_good = false;
        let mut instruction_address = 0;
        let mut initial_regs = Registers::V1(Default::default());

        // Repeatedly generate random registers until one set qualifies as valid.
        while !registers_good {
            initial_regs = match config.test_gen.cpu_type {
                MooCpuType::Intel80286 => {
                    let mut random_v2 = Registers::V2(RemoteCpuRegistersV2::default());
                    randomize_v2(context, config.test_gen.clone(), rng, &mut random_v2);

                    if config.test_exec.print_initial_regs {
                        print_regs(&random_v2, config.test_gen.cpu_type.into());
                    }
                    random_v2
                }
                _ => Registers::V1(RemoteCpuRegistersV1::default()),
            };

            if matches!(config.test_gen.cpu_mode, CpuMode::Real) {
                // Doing real mode test. Normalize the segment descriptors.
                initial_regs.normalize_descriptors();
            }

            // Check if the instruction is valid with the current registers.
            instruction_address =
                initial_regs.calculate_code_address() & config.test_gen.address_mask;
            if instruction_range.contains(&instruction_address) {
                registers_good = true;
            }
        }

        TestRegisters {
            regs: initial_regs,
            instruction_address,
        }
    }
}

pub fn randomize_v2(
    _context: &mut TestContext,
    config: TestGen,
    rng: &mut StdRng,
    regs: &mut Registers,
) {
    let random_opts = RandomizeOpts {
        weight_zero: config.reg_zero_chance,
        weight_ones: config.reg_ff_chance,
        weight_sp_odd: config.sp_odd_chance,
        sp_min_value: config.sp_min_value,
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
    };

    let mut reg_beta = Beta::new(config.register_beta[0], config.register_beta[1])
        .expect("Couldn't create beta function for register randomization");

    regs.randomize(&random_opts, rng, &mut reg_beta);
}
