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
    config::WeightedValue,
    cpu_common::{Register16, Register32},
};
use arduinox86_client::{
    registers_common::{RandomizeOpts, SegmentSize},
    Registers32,
};
use iced_x86::Mnemonic;
use moo::prelude::*;
use rand::distr::weighted::WeightedIndex;
use rand_distr::Beta;
use std::io::{Seek, Write};

#[derive(Debug)]
pub enum Registers {
    V1(arduinox86_client::RemoteCpuRegistersV1),
    V2(arduinox86_client::RemoteCpuRegistersV2),
    V3A(arduinox86_client::RemoteCpuRegistersV3A),
    V3B(arduinox86_client::RemoteCpuRegistersV3B),
}

impl TryFrom<&Registers> for MooRegisters {
    type Error = String;

    fn try_from(regs: &Registers) -> Result<Self, Self::Error> {
        match regs {
            Registers::V1(v1) => Ok(MooRegisters::Sixteen(MooRegisters16::from(v1))),
            Registers::V2(v2) => Ok(MooRegisters::Sixteen(MooRegisters16::from(v2))),
            Registers::V3A(v3a) => Ok(MooRegisters::ThirtyTwo(MooRegisters32::from(v3a))),
            Registers::V3B(v3b) => Ok(MooRegisters::ThirtyTwo(MooRegisters32::from(v3b))),
        }
    }
}

impl TryFrom<&Registers> for MooRegisters16 {
    type Error = String;

    fn try_from(regs: &Registers) -> Result<Self, Self::Error> {
        match regs {
            Registers::V1(v1) => Ok((&MooRegisters16Init {
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
            Registers::V2(v2) => Ok((&MooRegisters16Init {
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
            _ => Err("Unsupported register version for MooRegisters16 conversion".to_string()),
        }
    }
}

impl From<&Registers> for MooRegistersInit {
    fn from(regs: &Registers) -> Self {
        match regs {
            Registers::V1(v1) => MooRegistersInit::Sixteen(MooRegisters16Init {
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
            }),
            Registers::V2(v2) => MooRegistersInit::Sixteen(MooRegisters16Init {
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
            }),
            Registers::V3A(v3a) => MooRegistersInit::ThirtyTwo(MooRegisters32Init {
                cr0: v3a.cr0,
                cr3: 0,
                eax: v3a.eax,
                ebx: v3a.ebx,
                ecx: v3a.ecx,
                edx: v3a.edx,
                esi: v3a.esi,
                edi: v3a.edi,
                ebp: v3a.ebp,
                esp: v3a.esp,
                cs: v3a.cs as u32,
                ds: v3a.ds as u32,
                es: v3a.es as u32,
                fs: v3a.fs as u32,
                gs: v3a.gs as u32,
                ss: v3a.ss as u32,
                eip: v3a.eip,
                dr6: v3a.dr6,
                dr7: v3a.dr7,
                eflags: v3a.eflags,
            }),
            Registers::V3B(v3b) => MooRegistersInit::ThirtyTwo(MooRegisters32Init {
                cr0: v3b.cr0,
                cr3: v3b.cr3,
                eax: v3b.eax,
                ebx: v3b.ebx,
                ecx: v3b.ecx,
                edx: v3b.edx,
                esi: v3b.esi,
                edi: v3b.edi,
                ebp: v3b.ebp,
                esp: v3b.esp,
                cs: v3b.cs as u32,
                ds: v3b.ds as u32,
                es: v3b.es as u32,
                fs: v3b.fs as u32,
                gs: v3b.gs as u32,
                ss: v3b.ss as u32,
                eip: v3b.eip,
                dr6: v3b.dr6,
                dr7: v3b.dr7,
                eflags: v3b.eflags,
            }),
        }
    }
}

impl TryFrom<&Registers> for MooRegisters32 {
    type Error = String;

    fn try_from(regs: &Registers) -> Result<Self, Self::Error> {
        match regs {
            Registers::V3A(v3a) => Ok((&MooRegisters32Init {
                cr0: v3a.cr0,
                cr3: 0,
                eax: v3a.eax,
                ebx: v3a.ebx,
                ecx: v3a.ecx,
                edx: v3a.edx,
                esi: v3a.esi,
                edi: v3a.edi,
                ebp: v3a.ebp,
                esp: v3a.esp,
                cs: v3a.cs as u32,
                ds: v3a.ds as u32,
                es: v3a.es as u32,
                fs: v3a.fs as u32,
                gs: v3a.gs as u32,
                ss: v3a.ss as u32,
                eip: v3a.eip,
                dr6: v3a.dr6,
                dr7: v3a.dr7,
                eflags: v3a.eflags,
            })
                .into()),
            Registers::V3B(v3b) => Ok((&MooRegisters32Init {
                cr0: v3b.cr0,
                cr3: v3b.cr3,
                eax: v3b.eax,
                ebx: v3b.ebx,
                ecx: v3b.ecx,
                edx: v3b.edx,
                esi: v3b.esi,
                edi: v3b.edi,
                ebp: v3b.ebp,
                esp: v3b.esp,
                cs: v3b.cs as u32,
                ds: v3b.ds as u32,
                es: v3b.es as u32,
                fs: v3b.fs as u32,
                gs: v3b.gs as u32,
                ss: v3b.ss as u32,
                eip: v3b.eip,
                dr6: v3b.dr6,
                dr7: v3b.dr7,
                eflags: v3b.eflags,
            })
                .into()),
            _ => Err("Unsupported register version for MooRegisters32 conversion".to_string()),
        }
    }
}

impl Registers {
    pub const POP_FLAGS_MASK: u32 = 0b0000111011010111;

    pub fn randomize(
        &mut self,
        opts: &RandomizeOpts,
        rng: &mut rand::rngs::StdRng,
        beta: &mut Beta<f64>,
        weighted_index: &WeightedIndex<f32>,
        inject_values: &[u32],
    ) {
        match self {
            Registers::V1(_regs) => {
                //gen_regs::randomize_v1(&self.context, &self.config.test_gen, regs);
            }
            Registers::V2(regs) => regs.randomize(opts, rng, beta, weighted_index, inject_values),
            Registers::V3A(regs) => regs.randomize(opts, rng, beta, weighted_index, inject_values),
            Registers::V3B(_) => {
                // B registers don't need randomization as they are output
            }
        }
    }

    pub fn to_buffer<WS: Write + Seek>(&self, buf: &mut WS) {
        match self {
            Registers::V1(_regs) => {
                //gen_regs::write_v1(&mut W, regs);
                unimplemented!("Writing V1 registers to buffer is not implemented yet");
            }
            Registers::V2(regs) => regs.to_buffer(buf),
            Registers::V3A(regs) => _ = regs.to_buffer(buf),
            Registers::V3B(_regs) => {}
        }
    }

    pub fn buf_len(&self) -> usize {
        match self {
            Registers::V1(_regs) => 28,
            Registers::V2(_regs) => 102,
            Registers::V3A(_regs) => 204,
            Registers::V3B(_regs) => 208,
        }
    }

    pub fn calculate_code_address(&self) -> u32 {
        match self {
            Registers::V1(regs) => regs.calculate_code_address(),
            Registers::V2(regs) => regs.calculate_code_address(),
            Registers::V3A(regs) => regs.calculate_code_address(),
            Registers::V3B(regs) => regs.calculate_code_address(),
        }
    }

    pub fn normalize_descriptors(&mut self) {
        match self {
            Registers::V1(_regs) => {}
            Registers::V2(regs) => regs.normalize_descriptors(),
            Registers::V3A(regs) => regs.normalize_descriptors(),
            Registers::V3B(regs) => regs.normalize_descriptors(),
        }
    }

    pub fn ip(&self) -> u16 {
        match self {
            Registers::V1(regs) => regs.ip,
            Registers::V2(regs) => regs.ip,
            Registers::V3A(regs) => regs.eip as u16,
            Registers::V3B(regs) => regs.eip as u16,
        }
    }
    pub fn eip(&self) -> u32 {
        match self {
            Registers::V1(regs) => regs.ip as u32,
            Registers::V2(regs) => regs.ip as u32,
            Registers::V3A(regs) => regs.eip,
            Registers::V3B(regs) => regs.eip,
        }
    }
    pub fn cs(&self) -> u16 {
        match self {
            Registers::V1(regs) => regs.cs,
            Registers::V2(regs) => regs.cs,
            Registers::V3A(regs) => regs.cs,
            Registers::V3B(regs) => regs.cs,
        }
    }
    pub fn cs_base(&self) -> u32 {
        match self {
            Registers::V1(regs) => (regs.cs as u32) << 4,
            Registers::V2(regs) => regs.cs_desc.base_address(),
            Registers::V3A(regs) => regs.cs_desc.base_address(),
            Registers::V3B(regs) => regs.cs_desc.base_address(),
        }
    }
    pub fn ds_base(&self) -> u32 {
        match self {
            Registers::V1(regs) => (regs.ds as u32) << 4,
            Registers::V2(regs) => regs.ds_desc.base_address(),
            Registers::V3A(regs) => regs.ds_desc.base_address(),
            Registers::V3B(regs) => regs.ds_desc.base_address(),
        }
    }
    pub fn es_base(&self) -> u32 {
        match self {
            Registers::V1(regs) => (regs.es as u32) << 4,
            Registers::V2(regs) => regs.es_desc.base_address(),
            Registers::V3A(regs) => regs.es_desc.base_address(),
            Registers::V3B(regs) => regs.es_desc.base_address(),
        }
    }
    pub fn fs_base(&self) -> u32 {
        match self {
            Registers::V1(_regs) => 0,
            Registers::V2(_regs) => 0,
            Registers::V3A(regs) => regs.fs_desc.base_address(),
            Registers::V3B(regs) => regs.fs_desc.base_address(),
        }
    }
    pub fn gs_base(&self) -> u32 {
        match self {
            Registers::V1(_regs) => 0,
            Registers::V2(_regs) => 0,
            Registers::V3A(regs) => regs.gs_desc.base_address(),
            Registers::V3B(regs) => regs.gs_desc.base_address(),
        }
    }
    pub fn ss(&self) -> u16 {
        match self {
            Registers::V1(regs) => regs.ss,
            Registers::V2(regs) => regs.ss,
            Registers::V3A(regs) => regs.ss,
            Registers::V3B(regs) => regs.ss,
        }
    }
    pub fn ss_base(&self) -> u32 {
        match self {
            Registers::V1(regs) => (regs.ss as u32) << 4,
            Registers::V2(regs) => regs.ss_desc.base_address(),
            Registers::V3A(regs) => regs.ss_desc.base_address(),
            Registers::V3B(regs) => regs.ss_desc.base_address(),
        }
    }
    pub fn segment_limit(&self, segment: iced_x86::Register) -> Option<u32> {
        match self {
            Registers::V1(_regs) => None,
            Registers::V2(regs) => match segment {
                iced_x86::Register::DS => Some(regs.ds_desc.limit() as u32),
                iced_x86::Register::ES => Some(regs.es_desc.limit() as u32),
                iced_x86::Register::SS => Some(regs.ss_desc.limit() as u32),
                iced_x86::Register::CS => Some(regs.cs_desc.limit() as u32),
                _ => None,
            },
            Registers::V3A(regs) => match segment {
                iced_x86::Register::DS => Some(regs.ds_desc.limit()),
                iced_x86::Register::ES => Some(regs.es_desc.limit()),
                iced_x86::Register::FS => Some(regs.fs_desc.limit()),
                iced_x86::Register::GS => Some(regs.gs_desc.limit()),
                iced_x86::Register::SS => Some(regs.ss_desc.limit()),
                iced_x86::Register::CS => Some(regs.cs_desc.limit()),
                _ => None,
            },
            Registers::V3B(_regs) => None,
        }
    }
    pub fn segment_base(&self, segment: iced_x86::Register) -> Option<u32> {
        match self {
            Registers::V1(_regs) => None,
            Registers::V2(regs) => match segment {
                iced_x86::Register::DS => Some(regs.ds_desc.base_address()),
                iced_x86::Register::ES => Some(regs.es_desc.base_address()),
                iced_x86::Register::SS => Some(regs.ss_desc.base_address()),
                iced_x86::Register::CS => Some(regs.cs_desc.base_address()),
                _ => None,
            },
            Registers::V3A(regs) => match segment {
                iced_x86::Register::DS => Some(regs.ds_desc.base_address()),
                iced_x86::Register::ES => Some(regs.es_desc.base_address()),
                iced_x86::Register::FS => Some(regs.fs_desc.base_address()),
                iced_x86::Register::GS => Some(regs.gs_desc.base_address()),
                iced_x86::Register::SS => Some(regs.ss_desc.base_address()),
                iced_x86::Register::CS => Some(regs.cs_desc.base_address()),
                _ => None,
            },
            Registers::V3B(regs) => match segment {
                iced_x86::Register::DS => Some(regs.ds_desc.base_address()),
                iced_x86::Register::ES => Some(regs.es_desc.base_address()),
                iced_x86::Register::FS => Some(regs.fs_desc.base_address()),
                iced_x86::Register::GS => Some(regs.gs_desc.base_address()),
                iced_x86::Register::SS => Some(regs.ss_desc.base_address()),
                iced_x86::Register::CS => Some(regs.cs_desc.base_address()),
                _ => None,
            },
        }
    }

    pub fn segment_selector(&self, segment: iced_x86::Register) -> Option<u16> {
        match self {
            Registers::V1(_regs) => None,
            Registers::V2(regs) => match segment {
                iced_x86::Register::DS => Some(regs.ds),
                iced_x86::Register::ES => Some(regs.es),
                iced_x86::Register::SS => Some(regs.ss),
                iced_x86::Register::CS => Some(regs.cs),
                _ => None,
            },
            Registers::V3A(regs) => match segment {
                iced_x86::Register::DS => Some(regs.ds),
                iced_x86::Register::ES => Some(regs.es),
                iced_x86::Register::FS => Some(regs.fs),
                iced_x86::Register::GS => Some(regs.gs),
                iced_x86::Register::SS => Some(regs.ss),
                iced_x86::Register::CS => Some(regs.cs),
                _ => None,
            },
            Registers::V3B(regs) => match segment {
                iced_x86::Register::DS => Some(regs.ds),
                iced_x86::Register::ES => Some(regs.es),
                iced_x86::Register::FS => Some(regs.fs),
                iced_x86::Register::GS => Some(regs.gs),
                iced_x86::Register::SS => Some(regs.ss),
                iced_x86::Register::CS => Some(regs.cs),
                _ => None,
            },
        }
    }

    pub fn segment_size(&self, segment: iced_x86::Register) -> SegmentSize {
        match self {
            Registers::V1(_regs) => SegmentSize::Sixteen,
            Registers::V2(_regs) => SegmentSize::Sixteen,
            Registers::V3A(regs) => match segment {
                iced_x86::Register::DS => regs.ds_desc.segment_size(),
                iced_x86::Register::ES => regs.es_desc.segment_size(),
                iced_x86::Register::FS => regs.fs_desc.segment_size(),
                iced_x86::Register::GS => regs.gs_desc.segment_size(),
                iced_x86::Register::SS => regs.ss_desc.segment_size(),
                iced_x86::Register::CS => regs.cs_desc.segment_size(),
                _ => SegmentSize::Sixteen,
            },
            Registers::V3B(_regs) => unimplemented!("Segment size for V3B registers is not implemented"),
        }
    }
    pub fn cx(&self) -> u16 {
        match self {
            Registers::V1(regs) => regs.cx,
            Registers::V2(regs) => regs.cx,
            Registers::V3A(regs) => regs.ecx as u16,
            Registers::V3B(regs) => regs.ecx as u16,
        }
    }
    pub fn set_cx(&mut self, value: u16) {
        match self {
            Registers::V1(regs) => regs.cx = value,
            Registers::V2(regs) => regs.cx = value,
            Registers::V3A(regs) => regs.ecx = (regs.ecx & 0xFFFF_0000) | value as u32,
            Registers::V3B(regs) => regs.ecx = (regs.ecx & 0xFFFF_0000) | value as u32,
        }
    }
    pub fn ecx(&self) -> u32 {
        match self {
            Registers::V1(regs) => regs.cx as u32,
            Registers::V2(regs) => regs.cx as u32,
            Registers::V3A(regs) => regs.ecx,
            Registers::V3B(regs) => regs.ecx,
        }
    }
    pub fn set_ecx(&mut self, value: u32) {
        match self {
            Registers::V1(regs) => regs.cx = value as u16,
            Registers::V2(regs) => regs.cx = value as u16,
            Registers::V3A(regs) => regs.ecx = value,
            Registers::V3B(regs) => regs.ecx = value,
        }
    }
    pub fn dx(&self) -> u16 {
        match self {
            Registers::V1(regs) => regs.dx,
            Registers::V2(regs) => regs.dx,
            Registers::V3A(regs) => regs.edx as u16,
            Registers::V3B(regs) => regs.edx as u16,
        }
    }
    pub fn edx(&self) -> u32 {
        match self {
            Registers::V1(regs) => regs.dx as u32,
            Registers::V2(regs) => regs.dx as u32,
            Registers::V3A(regs) => regs.edx,
            Registers::V3B(regs) => regs.edx,
        }
    }
    pub fn sp(&self) -> u16 {
        match self {
            Registers::V1(regs) => regs.sp,
            Registers::V2(regs) => regs.sp,
            Registers::V3A(regs) => regs.esp as u16,
            Registers::V3B(regs) => regs.esp as u16,
        }
    }
    pub fn flags(&self) -> u16 {
        match self {
            Registers::V1(regs) => regs.flags,
            Registers::V2(regs) => regs.flags,
            Registers::V3A(regs) => regs.eflags as u16,
            Registers::V3B(regs) => regs.eflags as u16,
        }
    }
    pub fn stack_address(&self) -> u32 {
        match self {
            Registers::V1(regs) => ((regs.ss as u32) << 4).wrapping_add(regs.sp as u32),
            Registers::V2(regs) => regs.ss_desc.base_address().wrapping_add(regs.sp as u32),
            Registers::V3A(regs) => regs.ss_desc.base_address().wrapping_add(regs.esp),
            Registers::V3B(regs) => regs.ss_desc.base_address().wrapping_add(regs.esp),
        }
    }
    pub fn mask_registers32(
        &mut self,
        segment: iced_x86::Register,
        ea_registers: &[iced_x86::Register],
        mask_shift: u32,
    ) {
        match self {
            Registers::V1(_regs) => {}
            Registers::V2(_regs) => {}
            Registers::V3A(regs) => regs.mask_registers(segment, ea_registers, mask_shift),
            Registers::V3B(_regs) => {}
        }
    }

    pub fn segment_base16(&self, segment: Register16) -> u32 {
        match segment {
            Register16::CS => self.cs_base(),
            Register16::DS => self.ds_base(),
            Register16::ES => self.es_base(),
            Register16::SS => self.ss_base(),
            Register16::FS => self.fs_base(),
            Register16::GS => self.gs_base(),
            _ => 0,
        }
    }
    pub fn segment_base32(&self, segment: Register32) -> u32 {
        match segment {
            Register32::CS => self.cs_base(),
            Register32::DS => self.ds_base(),
            Register32::ES => self.es_base(),
            Register32::SS => self.ss_base(),
            Register32::FS => self.fs_base(),
            Register32::GS => self.gs_base(),
            _ => 0,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        let moo_registers = MooRegisters::try_from(self)
            .map_err(|e| anyhow::anyhow!("Failed to convert registers to MooRegisters: {}", e))?;

        // Check for reserved bit. Flags shouldn't be 0.
        let flags = moo_registers.flags();
        if flags & 0x0002 == 0 {
            // Reserved bit is not set.
            return Err(anyhow::anyhow!("Reserved bit in flags is not set: {:04X}", flags,));
        }

        Ok(())
    }
}

pub fn compare_registers(regs0: &MooRegisters, regs1: &MooRegisters) {
    match (regs0, regs1) {
        (MooRegisters::Sixteen(regs0_inner), MooRegisters::Sixteen(regs1_inner)) => {
            compare_registers16(regs0_inner, regs1_inner);
        }
        (MooRegisters::ThirtyTwo(regs0_inner), MooRegisters::ThirtyTwo(regs1_inner)) => {
            compare_registers32(regs0_inner, regs1_inner);
        }
        _ => {
            println!("Incompatible register types for comparison!");
        }
    }
}

pub fn compare_registers16(regs0: &MooRegisters16, regs1: &MooRegisters16) {
    if regs0.ax != regs1.ax {
        println!("AX mismatch: {:04X} != {:04X}", regs0.ax, regs1.ax);
    }
    if regs0.bx != regs1.bx {
        println!("BX mismatch: {:04X} != {:04X}", regs0.bx, regs1.bx);
    }
    if regs0.cx != regs1.cx {
        println!("CX mismatch: {:04X} != {:04X}", regs0.cx, regs1.cx);
    }
    if regs0.dx != regs1.dx {
        println!("DX mismatch: {:04X} != {:04X}", regs0.dx, regs1.dx);
    }
    if regs0.sp != regs1.sp {
        println!("SP mismatch: {:04X} != {:04X}", regs0.sp, regs1.sp);
    }
    if regs0.bp != regs1.bp {
        println!("BP mismatch: {:04X} != {:04X}", regs0.bp, regs1.bp);
    }
    if regs0.si != regs1.si {
        println!("SI mismatch: {:04X} != {:04X}", regs0.si, regs1.si);
    }
    if regs0.di != regs1.di {
        println!("DI mismatch: {:04X} != {:04X}", regs0.di, regs1.di);
    }
    if regs0.cs != regs1.cs {
        println!("CS mismatch: {:04X} != {:04X}", regs0.cs, regs1.cs);
    }
    if regs0.ds != regs1.ds {
        println!("DS mismatch: {:04X} != {:04X}", regs0.ds, regs1.ds);
    }
    if regs0.es != regs1.es {
        println!("ES mismatch: {:04X} != {:04X}", regs0.es, regs1.es);
    }
    if regs0.ss != regs1.ss {
        println!("SS mismatch: {:04X} != {:04X}", regs0.ss, regs1.ss);
    }
    if regs0.ip != regs1.ip {
        println!("IP mismatch: {:04X} != {:04X}", regs0.ip, regs1.ip);
    }
    if regs0.flags != regs1.flags {
        println!("FLAGS mismatch: {:04X} != {:04X}", regs0.flags, regs1.flags);
    }
}

pub fn compare_registers32(regs0: &MooRegisters32, regs1: &MooRegisters32) {
    match regs1.cr0() {
        Some(regs1_cr0) => {
            if regs0.cr0 != regs1_cr0 {
                println!("CR0 mismatch: {:08X} != {:08X}", regs0.cr0, regs1_cr0);
            }
        }
        None => {
            if regs0.cr0().is_some() {
                println!("CR0 mismatch: {:08X} != None", regs0.cr0);
            }
        }
    }

    if let Some(regs1_cr3) = regs1.cr3() {
        if regs0.cr3 != regs1_cr3 {
            println!("CR3 mismatch: {:08X} != {:08X}", regs0.cr3, regs1_cr3);
        }
    }
    if let Some(regs1_eax) = regs1.eax() {
        if regs0.eax != regs1_eax {
            println!("EAX mismatch: {:08X} != {:08X}", regs0.eax, regs1_eax);
        }
    }
    if let Some(regs1_ebx) = regs1.ebx() {
        if regs0.ebx != regs1_ebx {
            println!("EBX mismatch: {:08X} != {:08X}", regs0.ebx, regs1_ebx);
        }
    }
    if let Some(regs1_ecx) = regs1.ecx() {
        if regs0.ecx != regs1_ecx {
            println!("ECX mismatch: {:08X} != {:08X}", regs0.ecx, regs1_ecx);
        }
    }
    if let Some(regs1_edx) = regs1.edx() {
        if regs0.edx != regs1_edx {
            println!("EDX mismatch: {:08X} != {:08X}", regs0.edx, regs1_edx);
        }
    }
    if let Some(regs1_esp) = regs1.esp() {
        if regs0.esp != regs1_esp {
            println!("ESP mismatch: {:08X} != {:08X}", regs0.esp, regs1_esp);
        }
    }
    if let Some(regs1_ebp) = regs1.ebp() {
        if regs0.ebp != regs1_ebp {
            println!("EBP mismatch: {:08X} != {:08X}", regs0.ebp, regs1_ebp);
        }
    }
    if let Some(regs1_esi) = regs1.esi() {
        if regs0.esi != regs1_esi {
            println!("ESI mismatch: {:08X} != {:08X}", regs0.esi, regs1.esi);
        }
    }
    if let Some(regs1_edi) = regs1.edi() {
        if regs0.edi != regs1_edi {
            println!("EDI mismatch: {:08X} != {:08X}", regs0.edi, regs1.edi);
        }
    }
    if let Some(regs1_cs) = regs1.cs() {
        if regs0.cs != regs1_cs.into() {
            println!("CS mismatch: {:04X} != {:04X}", regs0.cs, regs1_cs);
        }
    }
    if let Some(regs1_ds) = regs1.ds() {
        if regs0.ds != regs1_ds.into() {
            println!("DS mismatch: {:04X} != {:04X}", regs0.ds, regs1_ds);
        }
    }
    if let Some(regs1_es) = regs1.es() {
        if regs0.es != regs1_es.into() {
            println!("ES mismatch: {:04X} != {:04X}", regs0.es, regs1_es);
        }
    }
    if let Some(regs1_fs) = regs1.fs() {
        if regs0.fs != regs1_fs.into() {
            println!("FS mismatch: {:04X} != {:04X}", regs0.fs, regs1_fs);
        }
    }
    if let Some(regs1_gs) = regs1.gs() {
        if regs0.gs != regs1_gs.into() {
            println!("GS mismatch: {:04X} != {:04X}", regs0.gs, regs1_gs);
        }
    }
    if let Some(regs1_ss) = regs1.ss() {
        if regs0.ss != regs1_ss.into() {
            println!("SS mismatch: {:04X} != {:04X}", regs0.ss, regs1_ss);
        }
    }
    if let Some(regs1_eip) = regs1.eip() {
        if regs0.eip != regs1_eip {
            println!("EIP mismatch: {:08X} != {:08X}", regs0.eip, regs1_eip);
        }
    }
    if let Some(regs1_eflags) = regs1.eflags() {
        if regs0.eflags != regs1_eflags {
            println!("EFLAGS mismatch: {:08X} != {:08X}", regs0.eflags, regs1_eflags);
        }
    }
    if let Some(regs1_dr6) = regs1.dr6() {
        if regs0.dr6 != regs1_dr6 {
            println!("DR6 mismatch: {:08X} != {:08X}", regs0.dr6, regs1_dr6);
        }
    }
    if let Some(regs1_dr7) = regs1.dr7() {
        if regs0.dr7 != regs1_dr7 {
            println!("DR7 mismatch: {:08X} != {:08X}", regs0.dr7, regs1_dr7);
        }
    }
}

pub fn validate_register_delta(
    mnemonic: Mnemonic,
    initial_regs: &Registers,
    final_regs: &Registers,
) -> anyhow::Result<()> {
    let moo_initial = MooRegisters::try_from(initial_regs)
        .map_err(|e| anyhow::anyhow!("Failed to convert initial registers: {}", e))?;
    let moo_final =
        MooRegisters::try_from(final_regs).map_err(|e| anyhow::anyhow!("Failed to convert final registers: {}", e))?;

    let mut error = false;

    if let (MooRegisters::Sixteen(moo_initial_i), MooRegisters::Sixteen(moo_final_i)) = (moo_initial, moo_final) {
        if !matches!(mnemonic, Mnemonic::Xchg) {
            if (moo_initial_i.ax != moo_initial_i.cx) && (moo_final_i.ax == moo_initial_i.cx) {
                error = true;
            }
            if (moo_initial_i.cx != moo_initial_i.dx) && (moo_final_i.cx == moo_initial_i.dx) {
                error = true;
            }
            if (moo_initial_i.dx != moo_initial_i.bx) && (moo_final_i.dx == moo_initial_i.bx) {
                error = true;
            }
            if (moo_initial_i.bx != moo_initial_i.sp) && (moo_final_i.bx == moo_initial_i.sp) {
                error = true;
            }
            if (moo_initial_i.sp != moo_initial_i.bp) && (moo_final_i.sp == moo_initial_i.bp) {
                error = true;
            }
            if (moo_initial_i.bp != moo_initial_i.si) && (moo_final_i.bp == moo_initial_i.si) {
                error = true;
            }
            if (moo_initial_i.si != moo_initial_i.di) && (moo_final_i.si == moo_initial_i.di) {
                error = true;
            }
            if (moo_initial_i.di != moo_initial_i.es) && (moo_final_i.di == moo_initial_i.es) {
                error = true;
            }
            if (moo_initial_i.es != moo_initial_i.cs) && (moo_final_i.es == moo_initial_i.cs) {
                error = true;
            }
            if (moo_initial_i.cs != moo_initial_i.ss) && (moo_final_i.cs == moo_initial_i.ss) {
                error = true;
            }
            if (moo_initial_i.ss != moo_initial_i.ds) && (moo_final_i.ss == moo_initial_i.ds) {
                error = true;
            }
        }
    }

    if error {
        log::error!("Possible off-by-one STOREALL register error detected!");
        return Err(anyhow::anyhow!("Possible off-by-one STOREALL register error detected!"));
    }
    Ok(())
}
