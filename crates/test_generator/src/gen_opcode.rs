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
use crate::{CpuType, TestContext, TestGen};
use anyhow::bail;
use rand::prelude::IndexedRandom;
use rand::Rng;
use std::collections::VecDeque;

macro_rules! get_rand {
    ($ctx: expr) => {
        $ctx.rng.gen()
    };
}

pub struct OpcodeGenerator {
    pub config: TestGen,
}

impl OpcodeGenerator {
    pub fn new(config: TestGen) -> Self {
        OpcodeGenerator { config }
    }

    pub fn generate(
        &mut self,
        context: &mut TestContext,
        opcode: u8,
        ext: Option<u8>,
    ) -> anyhow::Result<Vec<u8>> {
        let mut instruction_bytes = VecDeque::new();

        // Check opcode is valid.
        if self.config.excluded_opcodes.contains(&opcode) {
            bail!("Opcode {} is excluded from generation", opcode);
        }

        if self.config.prefixes.contains(&opcode) {
            bail!("Opcode {} is a prefix and cannot be generated", opcode);
        }

        // Of course we need the opcode itself...
        instruction_bytes.push_back(opcode);

        // Generate a random modrm.
        let modrm = get_rand!(context);

        // We can do specific filters on modrm values here if needed.
        match self.config.cpu_type {
            CpuType::Intel80286 => {
                // Any modrm is fine for the 80286 as invalid forms will generate a UD exception
                // instead of freaking out.
            }
            _ => {
                unimplemented!("Opcode generation for CPU type {:?}", self.config.cpu_type);
            }
        }

        // Push the modrm byte and six random bytes.
        instruction_bytes.push_back(modrm);
        for _ in 0..6 {
            let byte = get_rand!(context);
            instruction_bytes.push_back(byte);
        }

        // Roll for prefix type.
        let rand_u8: u8 = get_rand!(context);
        let prefix_type: u8 = rand_u8 % 2;

        match prefix_type {
            0 => {
                // Segment override prefix.
                // Roll percentage as specified in config.
                let rand_f32: f32 = get_rand!(context);
                if rand_f32 < self.config.segment_override_chance {
                    let segment_prefix = self
                        .config
                        .segment_prefixes
                        .choose(&mut context.rng)
                        .ok_or_else(|| anyhow::anyhow!("No segment prefixes defined!"))?;
                    instruction_bytes.push_front(*segment_prefix);
                }
            }
            1 => {
                // Lock prefix.
            }
            _ => {}
        }

        Ok(instruction_bytes.into())
    }
}
