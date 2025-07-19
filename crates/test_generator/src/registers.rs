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

use arduinox86_client::RandomizeOpts;
use moo::{prelude::MooRegisters1Init, types::MooRegisters1};
use rand_distr::Beta;
use std::io::Write;

pub enum Registers {
    V1(arduinox86_client::RemoteCpuRegistersV1),
    V2(arduinox86_client::RemoteCpuRegistersV2),
}

impl TryFrom<&Registers> for MooRegisters1 {
    type Error = String;

    fn try_from(regs: &Registers) -> Result<Self, Self::Error> {
        match regs {
            Registers::V1(v1) => Ok((&MooRegisters1Init {
                ax:    v1.ax,
                bx:    v1.bx,
                cx:    v1.cx,
                dx:    v1.dx,
                cs:    v1.cs,
                ss:    v1.ss,
                ds:    v1.ds,
                es:    v1.es,
                sp:    v1.sp,
                bp:    v1.bp,
                si:    v1.si,
                di:    v1.di,
                ip:    v1.ip,
                flags: v1.flags,
            })
                .into()),
            Registers::V2(v2) => Ok((&MooRegisters1Init {
                ax:    v2.ax,
                bx:    v2.bx,
                cx:    v2.cx,
                dx:    v2.dx,
                cs:    v2.cs,
                ss:    v2.ss,
                ds:    v2.ds,
                es:    v2.es,
                sp:    v2.sp,
                bp:    v2.bp,
                si:    v2.si,
                di:    v2.di,
                ip:    v2.ip,
                flags: v2.flags,
            })
                .into()),
        }
    }
}

impl From<&Registers> for MooRegisters1Init {
    fn from(regs: &Registers) -> Self {
        match regs {
            Registers::V1(v1) => MooRegisters1Init {
                ax:    v1.ax,
                bx:    v1.bx,
                cx:    v1.cx,
                dx:    v1.dx,
                cs:    v1.cs,
                ss:    v1.ss,
                ds:    v1.ds,
                es:    v1.es,
                sp:    v1.sp,
                bp:    v1.bp,
                si:    v1.si,
                di:    v1.di,
                ip:    v1.ip,
                flags: v1.flags,
            },
            Registers::V2(v2) => MooRegisters1Init {
                ax:    v2.ax,
                bx:    v2.bx,
                cx:    v2.cx,
                dx:    v2.dx,
                cs:    v2.cs,
                ss:    v2.ss,
                ds:    v2.ds,
                es:    v2.es,
                sp:    v2.sp,
                bp:    v2.bp,
                si:    v2.si,
                di:    v2.di,
                ip:    v2.ip,
                flags: v2.flags,
            },
        }
    }
}

impl Registers {
    pub fn randomize(&mut self, opts: &RandomizeOpts, rand: &mut rand::rngs::StdRng, beta: &mut Beta<f64>) {
        match self {
            Registers::V1(_regs) => {
                //gen_regs::randomize_v1(&self.context, &self.config.test_gen, regs);
            }
            Registers::V2(regs) => regs.randomize(opts, rand, beta),
        }
    }

    pub fn to_buffer<W: Write>(&self, buf: &mut W) {
        match self {
            Registers::V1(_regs) => {
                //gen_regs::write_v1(&mut W, regs);
                unimplemented!("Writing V1 registers to buffer is not implemented yet");
            }
            Registers::V2(regs) => regs.to_buffer(buf),
        }
    }

    pub fn buf_len(&self) -> usize {
        match self {
            Registers::V1(_regs) => 28,
            Registers::V2(_regs) => 102,
        }
    }

    pub fn calculate_code_address(&self) -> u32 {
        match self {
            Registers::V1(regs) => regs.calculate_code_address(),
            Registers::V2(regs) => regs.calculate_code_address(),
        }
    }

    pub fn normalize_descriptors(&mut self) {
        match self {
            Registers::V1(_regs) => {}
            Registers::V2(regs) => regs.normalize_descriptors(),
        }
    }

    pub fn ip(&self) -> u16 {
        match self {
            Registers::V1(regs) => regs.ip,
            Registers::V2(regs) => regs.ip,
        }
    }
    pub fn cs(&self) -> u16 {
        match self {
            Registers::V1(regs) => regs.cs,
            Registers::V2(regs) => regs.cs,
        }
    }
    pub fn cs_base(&self) -> u32 {
        match self {
            Registers::V1(regs) => regs.cs as u32,
            Registers::V2(regs) => regs.cs_desc.base_address(),
        }
    }
    pub fn cx(&self) -> u16 {
        match self {
            Registers::V1(regs) => regs.cx,
            Registers::V2(regs) => regs.cx,
        }
    }
    pub fn set_cx(&mut self, value: u16) {
        match self {
            Registers::V1(regs) => regs.cx = value,
            Registers::V2(regs) => regs.cx = value,
        }
    }
}
