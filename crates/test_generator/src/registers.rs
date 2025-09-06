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

use arduinox86_client::{
    registers_common::{RandomizeOpts, SegmentSize},
    Registers32,
};
use moo::{
    prelude::{MooRegisters16Init, MooRegisters32Init},
    types::{MooRegisters, MooRegisters16, MooRegisters32, MooRegistersInit},
};
use rand_distr::Beta;
use std::io::{Seek, Write};

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
    pub fn randomize(
        &mut self,
        opts: &RandomizeOpts,
        rand: &mut rand::rngs::StdRng,
        beta: &mut Beta<f64>,
        inject_values: &[u32],
    ) {
        match self {
            Registers::V1(_regs) => {
                //gen_regs::randomize_v1(&self.context, &self.config.test_gen, regs);
            }
            Registers::V2(regs) => regs.randomize(opts, rand, beta, inject_values),
            Registers::V3A(regs) => regs.randomize(opts, rand, beta, inject_values),
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
            Registers::V3B(regs) => {}
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
            Registers::V3B(regs) => None,
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
    pub fn sp(&self) -> u16 {
        match self {
            Registers::V1(regs) => regs.sp,
            Registers::V2(regs) => regs.sp,
            Registers::V3A(regs) => regs.esp as u16,
            Registers::V3B(regs) => regs.esp as u16,
        }
    }
    pub fn stack_address(&self) -> u32 {
        match self {
            Registers::V1(regs) => ((regs.ss as u32) << 4) + regs.sp as u32,
            Registers::V2(regs) => regs.ss_desc.base_address() + regs.sp as u32,
            Registers::V3A(regs) => regs.ss_desc.base_address() + regs.esp,
            Registers::V3B(regs) => regs.ss_desc.base_address() + regs.esp,
        }
    }
    pub fn mask_registers32(&mut self, segment: iced_x86::Register, ea_registers: &[iced_x86::Register]) {
        match self {
            Registers::V1(_regs) => {}
            Registers::V2(_regs) => {}
            Registers::V3A(regs) => regs.mask_registers(segment, ea_registers),
            Registers::V3B(_regs) => {}
        }
    }
}
