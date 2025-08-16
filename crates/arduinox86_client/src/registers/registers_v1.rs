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
use crate::RemoteCpuRegistersV2;
use moo::{prelude::MooRegisters16Init, types::MooRegisters16};

#[derive(Clone, Default, Debug)]
pub struct RemoteCpuRegistersV1 {
    pub ax:    u16,
    pub bx:    u16,
    pub cx:    u16,
    pub dx:    u16,
    pub ss:    u16,
    pub ds:    u16,
    pub es:    u16,
    pub sp:    u16,
    pub bp:    u16,
    pub si:    u16,
    pub di:    u16,
    pub cs:    u16,
    pub ip:    u16,
    pub flags: u16,
}

impl RemoteCpuRegistersV1 {
    pub const FLAG_CARRY: u16 = 0b0000_0000_0000_0001;
    pub const FLAG_RESERVED1: u16 = 0b0000_0000_0000_0010;
    pub const FLAG_PARITY: u16 = 0b0000_0000_0000_0100;
    pub const FLAG_RESERVED3: u16 = 0b0000_0000_0000_1000;
    pub const FLAG_AUX_CARRY: u16 = 0b0000_0000_0001_0000;
    pub const FLAG_RESERVED5: u16 = 0b0000_0000_0010_0000;
    pub const FLAG_ZERO: u16 = 0b0000_0000_0100_0000;
    pub const FLAG_SIGN: u16 = 0b0000_0000_1000_0000;
    pub const FLAG_TRAP: u16 = 0b0000_0001_0000_0000;
    pub const FLAG_INT_ENABLE: u16 = 0b0000_0010_0000_0000;
    pub const FLAG_DIRECTION: u16 = 0b0000_0100_0000_0000;
    pub const FLAG_OVERFLOW: u16 = 0b0000_1000_0000_0000;
    pub const FLAG_F15: u16 = 0b1000_0000_0000_0000; // Reserved bit 15
    pub const FLAG_MODE: u16 = 0b1000_0000_0000_0000;
    pub const FLAG_NT: u16 = 0b0100_0000_0000_0000; // Nested Task
    pub const FLAG_IOPL0: u16 = 0b0001_0000_0000_0000; // Nested Task
    pub const FLAG_IOPL1: u16 = 0b0010_0000_0000_0000; // Nested Task

    pub fn rewind_ip(&mut self, adjust: u16) {
        self.ip = self.ip.wrapping_sub(adjust);
    }

    pub fn write_buf(&self, buf: &mut [u8]) {
        // AX, BX, CX, DX, SS, SP, FLAGS, IP, CS, DS, ES, BP, SI, DI
        buf[0] = (self.ax & 0xFF) as u8;
        buf[1] = ((self.ax >> 8) & 0xFF) as u8;

        buf[2] = (self.bx & 0xFF) as u8;
        buf[3] = ((self.bx >> 8) & 0xFF) as u8;

        buf[4] = (self.cx & 0xFF) as u8;
        buf[5] = ((self.cx >> 8) & 0xFF) as u8;

        buf[6] = (self.dx & 0xFF) as u8;
        buf[7] = ((self.dx >> 8) & 0xFF) as u8;

        buf[8] = (self.ip & 0xFF) as u8;
        buf[9] = ((self.ip >> 8) & 0xFF) as u8;

        buf[10] = (self.cs & 0xFF) as u8;
        buf[11] = ((self.cs >> 8) & 0xFF) as u8;

        buf[12] = (self.flags & 0xFF) as u8;
        buf[13] = ((self.flags >> 8) & 0xFF) as u8;

        buf[14] = (self.ss & 0xFF) as u8;
        buf[15] = ((self.ss >> 8) & 0xFF) as u8;

        buf[16] = (self.sp & 0xFF) as u8;
        buf[17] = ((self.sp >> 8) & 0xFF) as u8;

        buf[18] = (self.ds & 0xFF) as u8;
        buf[19] = ((self.ds >> 8) & 0xFF) as u8;

        buf[20] = (self.es & 0xFF) as u8;
        buf[21] = ((self.es >> 8) & 0xFF) as u8;

        buf[22] = (self.bp & 0xFF) as u8;
        buf[23] = ((self.bp >> 8) & 0xFF) as u8;

        buf[24] = (self.si & 0xFF) as u8;
        buf[25] = ((self.si >> 8) & 0xFF) as u8;

        buf[26] = (self.di & 0xFF) as u8;
        buf[27] = ((self.di >> 8) & 0xFF) as u8;
    }

    pub fn calculate_code_address(&self) -> u32 {
        // Calculate the code address based on CS and IP
        ((self.cs as u32) << 4) + (self.ip as u32)
    }
}

impl From<&RemoteCpuRegistersV2> for RemoteCpuRegistersV1 {
    fn from(regs: &RemoteCpuRegistersV2) -> Self {
        RemoteCpuRegistersV1 {
            ax:    regs.ax,
            bx:    regs.bx,
            cx:    regs.cx,
            dx:    regs.dx,
            ss:    regs.ss,
            ds:    regs.ds,
            es:    regs.es,
            sp:    regs.sp,
            bp:    regs.bp,
            si:    regs.si,
            di:    regs.di,
            cs:    regs.cs,
            ip:    regs.ip,
            flags: regs.flags,
        }
    }
}

impl From<&[u8; 28]> for RemoteCpuRegistersV1 {
    fn from(buf: &[u8; 28]) -> Self {
        RemoteCpuRegistersV1 {
            ax:    buf[0] as u16 | ((buf[1] as u16) << 8),
            bx:    buf[2] as u16 | ((buf[3] as u16) << 8),
            cx:    buf[4] as u16 | ((buf[5] as u16) << 8),
            dx:    buf[6] as u16 | ((buf[7] as u16) << 8),
            ip:    buf[8] as u16 | ((buf[9] as u16) << 8),
            cs:    buf[10] as u16 | ((buf[11] as u16) << 8),
            flags: buf[12] as u16 | ((buf[13] as u16) << 8),
            ss:    buf[14] as u16 | ((buf[15] as u16) << 8),
            sp:    buf[16] as u16 | ((buf[17] as u16) << 8),
            ds:    buf[18] as u16 | ((buf[19] as u16) << 8),
            es:    buf[20] as u16 | ((buf[21] as u16) << 8),
            bp:    buf[22] as u16 | ((buf[23] as u16) << 8),
            si:    buf[24] as u16 | ((buf[25] as u16) << 8),
            di:    buf[26] as u16 | ((buf[27] as u16) << 8),
        }
    }
}
impl From<&[u8]> for RemoteCpuRegistersV1 {
    fn from(buf: &[u8]) -> Self {
        RemoteCpuRegistersV1 {
            ax:    buf[0] as u16 | ((buf[1] as u16) << 8),
            bx:    buf[2] as u16 | ((buf[3] as u16) << 8),
            cx:    buf[4] as u16 | ((buf[5] as u16) << 8),
            dx:    buf[6] as u16 | ((buf[7] as u16) << 8),
            ip:    buf[8] as u16 | ((buf[9] as u16) << 8),
            cs:    buf[10] as u16 | ((buf[11] as u16) << 8),
            flags: buf[12] as u16 | ((buf[13] as u16) << 8),
            ss:    buf[14] as u16 | ((buf[15] as u16) << 8),
            sp:    buf[16] as u16 | ((buf[17] as u16) << 8),
            ds:    buf[18] as u16 | ((buf[19] as u16) << 8),
            es:    buf[20] as u16 | ((buf[21] as u16) << 8),
            bp:    buf[22] as u16 | ((buf[23] as u16) << 8),
            si:    buf[24] as u16 | ((buf[25] as u16) << 8),
            di:    buf[26] as u16 | ((buf[27] as u16) << 8),
        }
    }
}

#[cfg(feature = "use_moo")]
impl From<RemoteCpuRegistersV1> for MooRegisters16 {
    fn from(remote: RemoteCpuRegistersV1) -> Self {
        MooRegisters16::from(&remote)
    }
}

#[cfg(feature = "use_moo")]
impl From<&RemoteCpuRegistersV1> for MooRegisters16 {
    fn from(remote: &RemoteCpuRegistersV1) -> Self {
        (&MooRegisters16Init {
            ax:    remote.ax,
            bx:    remote.bx,
            cx:    remote.cx,
            dx:    remote.dx,
            cs:    remote.cs,
            ss:    remote.ss,
            ds:    remote.ds,
            es:    remote.es,
            sp:    remote.sp,
            bp:    remote.bp,
            si:    remote.si,
            di:    remote.di,
            ip:    remote.ip,
            flags: remote.flags,
        })
            .into()
    }
}
