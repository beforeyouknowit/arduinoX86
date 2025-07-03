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
use binrw::BinReaderExt;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;

#[derive(Debug)]
pub enum RemoteCpuRegisters {
    V1(RemoteCpuRegistersV1),
    V2(RemoteCpuRegistersV2),
}

impl TryFrom<&[u8]> for RemoteCpuRegisters {
    type Error = &'static str;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        if buf.len() == 28 {
            Ok(RemoteCpuRegisters::V1(RemoteCpuRegistersV1::from(buf)))
        } else if buf.len() == 102 {
            Ok(RemoteCpuRegisters::V2(RemoteCpuRegistersV2::try_from(buf)?))
        } else {
            log::error!(
                "Invalid buffer length for RemoteCpuRegisters: {}",
                buf.len()
            );
            Err("Invalid buffer length for RemoteCpuRegisters!")
        }
    }
}

impl Default for RemoteCpuRegisters {
    fn default() -> Self {
        RemoteCpuRegisters::V1(RemoteCpuRegistersV1::default())
    }
}

impl RemoteCpuRegisters {
    pub fn set_cs(&mut self, cs: u16) {
        match self {
            RemoteCpuRegisters::V1(regs) => regs.cs = cs,
            RemoteCpuRegisters::V2(regs) => regs.cs = cs,
        }
    }

    pub fn set_ip(&mut self, ip: u16) {
        match self {
            RemoteCpuRegisters::V1(regs) => regs.ip = ip,
            RemoteCpuRegisters::V2(regs) => regs.ip = ip,
        }
    }

    pub fn rewind_ip(&mut self, adjust: u16) {
        match self {
            RemoteCpuRegisters::V1(regs) => regs.rewind_ip(adjust),
            RemoteCpuRegisters::V2(regs) => regs.rewind_ip(adjust),
        }
    }

    pub fn ax(&self) -> u16 {
        match self {
            RemoteCpuRegisters::V1(regs) => regs.ax,
            RemoteCpuRegisters::V2(regs) => regs.ax,
        }
    }

    pub fn flags(&self) -> u16 {
        match self {
            RemoteCpuRegisters::V1(regs) => regs.flags,
            RemoteCpuRegisters::V2(regs) => regs.flags,
        }
    }
}

#[derive(Default, Debug)]
pub struct RemoteCpuRegistersV1 {
    pub ax: u16,
    pub bx: u16,
    pub cx: u16,
    pub dx: u16,
    pub ss: u16,
    pub ds: u16,
    pub es: u16,
    pub sp: u16,
    pub bp: u16,
    pub si: u16,
    pub di: u16,
    pub cs: u16,
    pub ip: u16,
    pub flags: u16,
}

impl RemoteCpuRegistersV1 {
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
}

impl From<&[u8; 28]> for RemoteCpuRegistersV1 {
    fn from(buf: &[u8; 28]) -> Self {
        RemoteCpuRegistersV1 {
            ax: buf[0] as u16 | ((buf[1] as u16) << 8),
            bx: buf[2] as u16 | ((buf[3] as u16) << 8),
            cx: buf[4] as u16 | ((buf[5] as u16) << 8),
            dx: buf[6] as u16 | ((buf[7] as u16) << 8),
            ip: buf[8] as u16 | ((buf[9] as u16) << 8),
            cs: buf[10] as u16 | ((buf[11] as u16) << 8),
            flags: buf[12] as u16 | ((buf[13] as u16) << 8),
            ss: buf[14] as u16 | ((buf[15] as u16) << 8),
            sp: buf[16] as u16 | ((buf[17] as u16) << 8),
            ds: buf[18] as u16 | ((buf[19] as u16) << 8),
            es: buf[20] as u16 | ((buf[21] as u16) << 8),
            bp: buf[22] as u16 | ((buf[23] as u16) << 8),
            si: buf[24] as u16 | ((buf[25] as u16) << 8),
            di: buf[26] as u16 | ((buf[27] as u16) << 8),
        }
    }
}
impl From<&[u8]> for RemoteCpuRegistersV1 {
    fn from(buf: &[u8]) -> Self {
        RemoteCpuRegistersV1 {
            ax: buf[0] as u16 | ((buf[1] as u16) << 8),
            bx: buf[2] as u16 | ((buf[3] as u16) << 8),
            cx: buf[4] as u16 | ((buf[5] as u16) << 8),
            dx: buf[6] as u16 | ((buf[7] as u16) << 8),
            ip: buf[8] as u16 | ((buf[9] as u16) << 8),
            cs: buf[10] as u16 | ((buf[11] as u16) << 8),
            flags: buf[12] as u16 | ((buf[13] as u16) << 8),
            ss: buf[14] as u16 | ((buf[15] as u16) << 8),
            sp: buf[16] as u16 | ((buf[17] as u16) << 8),
            ds: buf[18] as u16 | ((buf[19] as u16) << 8),
            es: buf[20] as u16 | ((buf[21] as u16) << 8),
            bp: buf[22] as u16 | ((buf[23] as u16) << 8),
            si: buf[24] as u16 | ((buf[25] as u16) << 8),
            di: buf[26] as u16 | ((buf[27] as u16) << 8),
        }
    }
}

#[bitfield]
#[derive(Default, Debug)]
pub struct SegmentDescriptorV1 {
    pub address: B24,
    pub d_type: B4,
    pub s: B1,
    pub dpl: B2,
    pub p: B1,
    pub limit: B16,
}

/// [RemoteCpuRegistersV2] is the full set of registers for the Intel 80286.
/// This structure is loaded via the LOADALL instruction, 0F 05.
#[derive(Default, Debug)]
pub struct RemoteCpuRegistersV2 {
    pub _unused0: u16,
    pub _unused1: u16,
    pub msw: u16,
    pub _unused2: u16,
    pub _unused3: u16,
    pub _unused4: u16,
    pub _unused5: u16,
    pub _unused6: u16,
    pub _unused7: u16,
    pub _unused8: u16,
    pub _unused9: u16,
    pub tr: u16,
    pub flags: u16,
    pub ip: u16,
    pub ldt: u16,
    pub ds: u16,
    pub ss: u16,
    pub cs: u16,
    pub es: u16,
    pub di: u16,
    pub si: u16,
    pub bp: u16,
    pub sp: u16,
    pub bx: u16,
    pub dx: u16,
    pub cx: u16,
    pub ax: u16,
    pub es_desc: SegmentDescriptorV1,
    pub cs_desc: SegmentDescriptorV1,
    pub ss_desc: SegmentDescriptorV1,
    pub ds_desc: SegmentDescriptorV1,
    pub gdt_desc: SegmentDescriptorV1,
    pub ldt_desc: SegmentDescriptorV1,
    pub idt_desc: SegmentDescriptorV1,
    pub tss_desc: SegmentDescriptorV1,
}

impl TryFrom<&[u8]> for RemoteCpuRegistersV2 {
    type Error = &'static str;
    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        parse_v2(&buf)
    }
}

impl From<[u8; 102]> for RemoteCpuRegistersV2 {
    fn from(buf: [u8; 102]) -> RemoteCpuRegistersV2 {
        parse_v2(&buf).unwrap()
    }
}

fn parse_v2(buf: &[u8]) -> Result<RemoteCpuRegistersV2, &'static str> {
    if buf.len() < 102 {
        return Err("Buffer too small for RemoteCpuRegistersV2");
    }

    let mut new_regs = RemoteCpuRegistersV2::default();
    let mut cursor = std::io::Cursor::new(buf);

    new_regs._unused0 = cursor.read_le().unwrap();
    new_regs._unused1 = cursor.read_le().unwrap();
    new_regs.msw = cursor.read_le().unwrap();
    new_regs._unused2 = cursor.read_le().unwrap();
    new_regs._unused3 = cursor.read_le().unwrap();
    new_regs._unused4 = cursor.read_le().unwrap();
    new_regs._unused5 = cursor.read_le().unwrap();
    new_regs._unused6 = cursor.read_le().unwrap();
    new_regs._unused7 = cursor.read_le().unwrap();
    new_regs._unused8 = cursor.read_le().unwrap();
    new_regs._unused9 = cursor.read_le().unwrap();
    new_regs.tr = cursor.read_le().unwrap();
    new_regs.flags = cursor.read_le().unwrap();
    new_regs.ip = cursor.read_le().unwrap();
    new_regs.ldt = cursor.read_le().unwrap();
    new_regs.ds = cursor.read_le().unwrap();
    new_regs.ss = cursor.read_le().unwrap();
    new_regs.cs = cursor.read_le().unwrap();
    new_regs.es = cursor.read_le().unwrap();
    new_regs.di = cursor.read_le().unwrap();
    new_regs.si = cursor.read_le().unwrap();
    new_regs.bp = cursor.read_le().unwrap();
    new_regs.sp = cursor.read_le().unwrap();
    new_regs.bx = cursor.read_le().unwrap();
    new_regs.dx = cursor.read_le().unwrap();
    new_regs.cx = cursor.read_le().unwrap();
    new_regs.ax = cursor.read_le().unwrap();

    let idx = cursor.position();
    let desc_slice = &cursor.into_inner()[idx as usize..idx as usize + 48];

    new_regs.es_desc = read_desc(desc_slice, 0);
    new_regs.cs_desc = read_desc(desc_slice, 1);
    new_regs.ss_desc = read_desc(desc_slice, 2);
    new_regs.ds_desc = read_desc(desc_slice, 3);
    new_regs.gdt_desc = read_desc(desc_slice, 4);
    new_regs.ldt_desc = read_desc(desc_slice, 5);
    new_regs.idt_desc = read_desc(desc_slice, 6);
    new_regs.tss_desc = read_desc(desc_slice, 7);

    Ok(new_regs)
}

fn read_desc(slice: &[u8], index: usize) -> SegmentDescriptorV1 {
    // each descriptor is 6 bytes
    let start = index * 6;
    let end = start + 6;
    let bytes: [u8; 6] = slice[start..end]
        .try_into()
        .expect("desc_slice must be at least 6*8=48 bytes");
    SegmentDescriptorV1::from_bytes(bytes)
}

impl RemoteCpuRegistersV2 {
    pub fn rewind_ip(&mut self, adjust: u16) {
        self.ip = self.ip.wrapping_sub(adjust);
    }
}
