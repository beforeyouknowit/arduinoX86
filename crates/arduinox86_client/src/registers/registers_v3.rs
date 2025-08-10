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

use crate::Registers32;

use binrw::{binrw, BinReaderExt};

macro_rules! impl_registers32 {
    ($ty:ty $(,)?) => {
        impl Registers32 for $ty {
            // --- control/debug ---
            fn cr0(&self) -> u32 {
                self.cr0
            }
            // fn cr3(&self) -> u32 {
            //     self.cr3
            // }
            fn dr6(&self) -> u32 {
                self.dr6
            }
            fn dr7(&self) -> u32 {
                self.dr7
            }

            fn cr0_mut(&mut self) -> &mut u32 {
                &mut self.cr0
            }
            // fn cr3_mut(&mut self) -> &mut u32 {
            //     &mut self.cr3
            // }
            fn dr6_mut(&mut self) -> &mut u32 {
                &mut self.dr6
            }
            fn dr7_mut(&mut self) -> &mut u32 {
                &mut self.dr7
            }

            fn set_cr0(&mut self, value: u32) {
                self.cr0 = value;
            }
            // fn set_cr3(&mut self, value: u32) {
            //     self.cr3 = value;
            // }
            fn set_dr6(&mut self, value: u32) {
                self.dr6 = value;
            }
            fn set_dr7(&mut self, value: u32) {
                self.dr7 = value;
            }

            // --- GPRs ---
            fn eax(&self) -> u32 {
                self.eax
            }
            fn ebx(&self) -> u32 {
                self.ebx
            }
            fn ecx(&self) -> u32 {
                self.ecx
            }
            fn edx(&self) -> u32 {
                self.edx
            }
            fn esp(&self) -> u32 {
                self.esp
            }
            fn ebp(&self) -> u32 {
                self.ebp
            }
            fn esi(&self) -> u32 {
                self.esi
            }
            fn edi(&self) -> u32 {
                self.edi
            }
            fn eip(&self) -> u32 {
                self.eip
            }
            fn eflags(&self) -> u32 {
                self.eflags
            }

            fn eax_mut(&mut self) -> &mut u32 {
                &mut self.eax
            }
            fn ebx_mut(&mut self) -> &mut u32 {
                &mut self.ebx
            }
            fn ecx_mut(&mut self) -> &mut u32 {
                &mut self.ecx
            }
            fn edx_mut(&mut self) -> &mut u32 {
                &mut self.edx
            }
            fn esp_mut(&mut self) -> &mut u32 {
                &mut self.esp
            }
            fn ebp_mut(&mut self) -> &mut u32 {
                &mut self.ebp
            }
            fn esi_mut(&mut self) -> &mut u32 {
                &mut self.esi
            }
            fn edi_mut(&mut self) -> &mut u32 {
                &mut self.edi
            }
            fn eip_mut(&mut self) -> &mut u32 {
                &mut self.eip
            }
            fn eflags_mut(&mut self) -> &mut u32 {
                &mut self.eflags
            }

            fn set_eax(&mut self, value: u32) {
                self.eax = value;
            }
            fn set_ebx(&mut self, value: u32) {
                self.ebx = value;
            }
            fn set_ecx(&mut self, value: u32) {
                self.ecx = value;
            }
            fn set_edx(&mut self, value: u32) {
                self.edx = value;
            }
            fn set_esp(&mut self, value: u32) {
                self.esp = value;
            }
            fn set_ebp(&mut self, value: u32) {
                self.ebp = value;
            }
            fn set_esi(&mut self, value: u32) {
                self.esi = value;
            }
            fn set_edi(&mut self, value: u32) {
                self.edi = value;
            }
            fn set_eip(&mut self, value: u32) {
                self.eip = value;
            }
            fn set_eflags(&mut self, value: u32) {
                self.eflags = value;
            }

            // --- segments ---
            fn cs(&self) -> u16 {
                self.cs
            }
            fn ds(&self) -> u16 {
                self.ds
            }
            fn es(&self) -> u16 {
                self.es
            }
            fn fs(&self) -> u16 {
                self.fs
            }
            fn gs(&self) -> u16 {
                self.gs
            }
            fn ss(&self) -> u16 {
                self.ss
            }

            fn cs_mut(&mut self) -> &mut u16 {
                &mut self.cs
            }
            fn ds_mut(&mut self) -> &mut u16 {
                &mut self.ds
            }
            fn es_mut(&mut self) -> &mut u16 {
                &mut self.es
            }
            fn fs_mut(&mut self) -> &mut u16 {
                &mut self.fs
            }
            fn gs_mut(&mut self) -> &mut u16 {
                &mut self.gs
            }
            fn ss_mut(&mut self) -> &mut u16 {
                &mut self.ss
            }

            fn set_cs(&mut self, value: u16) {
                self.cs = value;
            }
            fn set_ds(&mut self, value: u16) {
                self.ds = value;
            }
            fn set_es(&mut self, value: u16) {
                self.es = value;
            }
            fn set_fs(&mut self, value: u16) {
                self.fs = value;
            }
            fn set_gs(&mut self, value: u16) {
                self.gs = value;
            }
            fn set_ss(&mut self, value: u16) {
                self.ss = value;
            }
        }
    };
}

#[derive(Clone)]
pub enum RemoteCpuRegistersV3 {
    A(RemoteCpuRegistersV3A),
    B(RemoteCpuRegistersV3B),
}

// value-returning getter
macro_rules! enum_get {
    ($Trait:path, $method:ident -> $ret:ty, $trait_m:ident) => {
        fn $method(&self) -> $ret {
            use $Trait as _; // bring trait into scope without a name
            match self {
                Self::A(r) => r.$trait_m(),
                Self::B(r) => r.$trait_m(),
            }
        }
    };
}

// &mut-returning getter
macro_rules! enum_get_mut {
    ($Trait:path, $method:ident -> &mut $ret:ty, $trait_m:ident) => {
        fn $method(&mut self) -> &mut $ret {
            use $Trait as _;
            match self {
                Self::A(r) => r.$trait_m(),
                Self::B(r) => r.$trait_m(),
            }
        }
    };
}

// setter
macro_rules! enum_set {
    ($Trait:path, $method:ident($arg_ty:ty) => $trait_m:ident) => {
        fn $method(&mut self, value: $arg_ty) {
            use $Trait as _;
            match self {
                Self::A(r) => r.$trait_m(value),
                Self::B(r) => r.$trait_m(value),
            }
        }
    };
}

impl Registers32 for RemoteCpuRegistersV3 {
    // control/debug
    enum_get!(crate::registers::Registers32, cr0 -> u32, cr0);
    //enum_get!(crate::registers::Registers32, cr3 -> u32, cr3);
    enum_get!(crate::registers::Registers32, dr6 -> u32, dr6);
    enum_get!(crate::registers::Registers32, dr7 -> u32, dr7);

    enum_get_mut!(crate::registers::Registers32, cr0_mut -> &mut u32, cr0_mut);
    //enum_get_mut!(crate::registers::Registers32, cr3_mut -> &mut u32, cr3_mut);
    enum_get_mut!(crate::registers::Registers32, dr6_mut -> &mut u32, dr6_mut);
    enum_get_mut!(crate::registers::Registers32, dr7_mut -> &mut u32, dr7_mut);

    enum_set!(crate::registers::Registers32, set_cr0(u32) => set_cr0);
    //enum_set!(crate::registers::Registers32, set_cr3(u32) => set_cr3);
    enum_set!(crate::registers::Registers32, set_dr6(u32) => set_dr6);
    enum_set!(crate::registers::Registers32, set_dr7(u32) => set_dr7);

    // gprs / ip / flags
    enum_get!(crate::registers::Registers32, eax -> u32, eax);
    enum_get!(crate::registers::Registers32, ebx -> u32, ebx);
    enum_get!(crate::registers::Registers32, ecx -> u32, ecx);
    enum_get!(crate::registers::Registers32, edx -> u32, edx);
    enum_get!(crate::registers::Registers32, esp -> u32, esp);
    enum_get!(crate::registers::Registers32, ebp -> u32, ebp);
    enum_get!(crate::registers::Registers32, esi -> u32, esi);
    enum_get!(crate::registers::Registers32, edi -> u32, edi);
    enum_get!(crate::registers::Registers32, eip -> u32, eip);
    enum_get!(crate::registers::Registers32, eflags -> u32, eflags);

    enum_get_mut!(crate::registers::Registers32, eax_mut -> &mut u32, eax_mut);
    enum_get_mut!(crate::registers::Registers32, ebx_mut -> &mut u32, ebx_mut);
    enum_get_mut!(crate::registers::Registers32, ecx_mut -> &mut u32, ecx_mut);
    enum_get_mut!(crate::registers::Registers32, edx_mut -> &mut u32, edx_mut);
    enum_get_mut!(crate::registers::Registers32, esp_mut -> &mut u32, esp_mut);
    enum_get_mut!(crate::registers::Registers32, ebp_mut -> &mut u32, ebp_mut);
    enum_get_mut!(crate::registers::Registers32, esi_mut -> &mut u32, esi_mut);
    enum_get_mut!(crate::registers::Registers32, edi_mut -> &mut u32, edi_mut);
    enum_get_mut!(crate::registers::Registers32, eip_mut -> &mut u32, eip_mut);
    enum_get_mut!(crate::registers::Registers32, eflags_mut -> &mut u32, eflags_mut);

    enum_set!(crate::registers::Registers32, set_eax(u32) => set_eax);
    enum_set!(crate::registers::Registers32, set_ebx(u32) => set_ebx);
    enum_set!(crate::registers::Registers32, set_ecx(u32) => set_ecx);
    enum_set!(crate::registers::Registers32, set_edx(u32) => set_edx);
    enum_set!(crate::registers::Registers32, set_esp(u32) => set_esp);
    enum_set!(crate::registers::Registers32, set_ebp(u32) => set_ebp);
    enum_set!(crate::registers::Registers32, set_esi(u32) => set_esi);
    enum_set!(crate::registers::Registers32, set_edi(u32) => set_edi);
    enum_set!(crate::registers::Registers32, set_eip(u32) => set_eip);
    enum_set!(crate::registers::Registers32, set_eflags(u32) => set_eflags);

    // segments
    enum_get!(crate::registers::Registers32, cs -> u16, cs);
    enum_get!(crate::registers::Registers32, ds -> u16, ds);
    enum_get!(crate::registers::Registers32, es -> u16, es);
    enum_get!(crate::registers::Registers32, fs -> u16, fs);
    enum_get!(crate::registers::Registers32, gs -> u16, gs);
    enum_get!(crate::registers::Registers32, ss -> u16, ss);

    enum_get_mut!(crate::registers::Registers32, cs_mut -> &mut u16, cs_mut);
    enum_get_mut!(crate::registers::Registers32, ds_mut -> &mut u16, ds_mut);
    enum_get_mut!(crate::registers::Registers32, es_mut -> &mut u16, es_mut);
    enum_get_mut!(crate::registers::Registers32, fs_mut -> &mut u16, fs_mut);
    enum_get_mut!(crate::registers::Registers32, gs_mut -> &mut u16, gs_mut);
    enum_get_mut!(crate::registers::Registers32, ss_mut -> &mut u16, ss_mut);

    enum_set!(crate::registers::Registers32, set_cs(u16) => set_cs);
    enum_set!(crate::registers::Registers32, set_ds(u16) => set_ds);
    enum_set!(crate::registers::Registers32, set_es(u16) => set_es);
    enum_set!(crate::registers::Registers32, set_fs(u16) => set_fs);
    enum_set!(crate::registers::Registers32, set_gs(u16) => set_gs);
    enum_set!(crate::registers::Registers32, set_ss(u16) => set_ss);
}

impl Default for RemoteCpuRegistersV3 {
    fn default() -> Self {
        RemoteCpuRegistersV3::A(RemoteCpuRegistersV3A::default())
    }
}

impl TryFrom<&[u8]> for RemoteCpuRegistersV3 {
    type Error = &'static str;
    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        match buf.len() {
            204 => Ok(RemoteCpuRegistersV3::A(RemoteCpuRegistersV3A::try_from(&buf[..])?)),
            208 => Ok(RemoteCpuRegistersV3::B(RemoteCpuRegistersV3B::try_from(&buf[..])?)),
            _ => Err("Invalid buffer length for RemoteCpuRegistersV3"),
        }
    }
}

impl RemoteCpuRegistersV3 {
    pub const FLAG_CARRY: u32 = 0b0000_0000_0000_0001;
    pub const FLAG_RESERVED1: u32 = 0b0000_0000_0000_0010;
    pub const FLAG_PARITY: u32 = 0b0000_0000_0000_0100;
    pub const FLAG_RESERVED3: u32 = 0b0000_0000_0000_1000;
    pub const FLAG_AUX_CARRY: u32 = 0b0000_0000_0001_0000;
    pub const FLAG_RESERVED5: u32 = 0b0000_0000_0010_0000;
    pub const FLAG_ZERO: u32 = 0b0000_0000_0100_0000;
    pub const FLAG_SIGN: u32 = 0b0000_0000_1000_0000;
    pub const FLAG_TRAP: u32 = 0b0000_0001_0000_0000;
    pub const FLAG_INT_ENABLE: u32 = 0b0000_0010_0000_0000;
    pub const FLAG_DIRECTION: u32 = 0b0000_0100_0000_0000;
    pub const FLAG_OVERFLOW: u32 = 0b0000_1000_0000_0000;
    pub const FLAG_F15: u32 = 0b1000_0000_0000_0000; // Reserved bit 15
    pub const FLAG_MODE: u32 = 0b1000_0000_0000_0000;
    pub const FLAG_NT: u32 = 0b0100_0000_0000_0000; // Nested Task
    pub const FLAG_IOPL0: u32 = 0b0001_0000_0000_0000; // Nested Task
    pub const FLAG_IOPL1: u32 = 0b0010_0000_0000_0000; // Nested Task

    pub fn calculate_code_address(&self) -> u32 {
        match self {
            RemoteCpuRegistersV3::A(regs) => regs.calculate_code_address(),
            RemoteCpuRegistersV3::B(regs) => regs.calculate_code_address(),
        }
    }
    pub fn set_cs(&mut self, cs: u16) {
        match self {
            RemoteCpuRegistersV3::A(regs) => regs.cs = cs,
            RemoteCpuRegistersV3::B(regs) => regs.cs = cs,
        }
    }
}

#[binrw]
#[brw(little)]
#[derive(Clone, Default)]
pub struct SegmentDescriptorV2 {
    pub access:  u32,
    pub address: u32,
    pub limit:   u32,
}

impl SegmentDescriptorV2 {
    pub fn with_base_address(mut self, base: u32) -> Self {
        self.address = base;
        self
    }
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }
    pub fn with_access(mut self, access: u32) -> Self {
        self.access = access;
        self
    }

    pub fn from_slice(slice: &[u8], index: usize) -> Self {
        read_descriptor_v2(slice, index)
    }
}

fn read_descriptor_v2(slice: &[u8], index: usize) -> SegmentDescriptorV2 {
    // Each descriptor is 12 bytes
    let start = index * 12;
    let end = start + 12;
    let bytes: [u8; 12] = slice[start..end]
        .try_into()
        .expect("desc_slice must be at least 10*12=120 bytes");
    SegmentDescriptorV2 {
        access:  u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        address: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        limit:   u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
    }
}

/// [RemoteCpuRegistersV3] is the LOADALL structure for the Intel 386.
/// This structure is loaded via the LOADALL instruction, 0F 05.
#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct RemoteCpuRegistersV3A {
    pub cr0: u32,    // +00
    pub eflags: u32, // +04
    pub eip: u32,    // +08
    pub edi: u32,    // +0C
    pub esi: u32,    // +10
    pub ebp: u32,    // +14
    pub esp: u32,    // +18
    pub ebx: u32,    // +1C
    pub edx: u32,    // +20
    pub ecx: u32,    // +24
    pub eax: u32,    // +28
    pub dr6: u32,    // +2C
    pub dr7: u32,    // +30
    pub tr: u16,     // +34
    pub tr_pad: u16,
    pub ldt: u16, // +38
    pub ldt_pad: u16,
    pub gs: u16, // +3C
    pub gs_pad: u16,
    pub fs: u16, // +40
    pub fs_pad: u16,
    pub ds: u16, // +44
    pub ds_pad: u16,
    pub ss: u16, // +48
    pub ss_pad: u16,
    pub cs: u16, // +4C
    pub cs_pad: u16,
    pub es: u16, // +50
    pub es_pad: u16,
    pub tss_desc: SegmentDescriptorV2,
    pub idt_desc: SegmentDescriptorV2,
    pub gdt_desc: SegmentDescriptorV2,
    pub ldt_desc: SegmentDescriptorV2,
    pub gs_desc: SegmentDescriptorV2,
    pub fs_desc: SegmentDescriptorV2,
    pub ds_desc: SegmentDescriptorV2,
    pub ss_desc: SegmentDescriptorV2,
    pub cs_desc: SegmentDescriptorV2,
    pub es_desc: SegmentDescriptorV2,
}

impl Default for RemoteCpuRegistersV3A {
    fn default() -> Self {
        RemoteCpuRegistersV3A {
            cr0: 0,
            eflags: 0x00000002, // reserved bit 1 set
            eip: 0x0000_0100,
            edi: 0,
            esi: 0,
            ebp: 0,
            esp: 0,
            ebx: 0,
            edx: 0,
            ecx: 0,
            eax: 0,
            dr6: 0,
            dr7: 0,
            tr: 0,
            tr_pad: 0,
            ldt: 0,
            ldt_pad: 0,
            gs: 0,
            gs_pad: 0,
            fs: 0,
            fs_pad: 0,
            ds: 0,
            ds_pad: 0,
            ss: 0,
            ss_pad: 0,
            cs: RemoteCpuRegistersV3A::DEFAULT_CS,
            cs_pad: 0,
            es: 0,
            es_pad: 0,

            // Default access values provided by Robert Collins
            // https://www.rcollins.org/ftp/source/386load/386load.asm
            tss_desc: SegmentDescriptorV2::default()
                .with_access(0x00008900)
                .with_limit(0xFFFFFFFF),
            idt_desc: SegmentDescriptorV2::default()
                .with_access(0x00008000)
                .with_limit(0xFFFFFFFF),
            gdt_desc: SegmentDescriptorV2::default()
                .with_access(0x00008000)
                .with_limit(0xFFFFFFFF),
            ldt_desc: SegmentDescriptorV2::default()
                .with_access(0x00008200)
                .with_limit(0xFFFFFFFF),
            gs_desc:  SegmentDescriptorV2::default()
                .with_access(0x00009300)
                .with_limit(0xFFFF),
            fs_desc:  SegmentDescriptorV2::default()
                .with_access(0x00009300)
                .with_limit(0xFFFF),
            ds_desc:  SegmentDescriptorV2::default()
                .with_access(0x00009300)
                .with_limit(0xFFFF),
            ss_desc:  SegmentDescriptorV2::default()
                .with_access(0x00009300)
                .with_limit(0xFFFF),
            cs_desc:  SegmentDescriptorV2::default()
                .with_base_address((RemoteCpuRegistersV3A::DEFAULT_CS as u32) << 4)
                .with_access(0x00009B00)
                .with_limit(0xFFFF),
            es_desc:  SegmentDescriptorV2::default()
                .with_access(0x00009300)
                .with_limit(0xFFFF),
        }
    }
}

impl RemoteCpuRegistersV3A {
    pub const DEFAULT_CS: u16 = 0x1000;

    pub const FLAG_CARRY: u32 = 0b0000_0000_0000_0001;
    pub const FLAG_RESERVED1: u32 = 0b0000_0000_0000_0010;
    pub const FLAG_PARITY: u32 = 0b0000_0000_0000_0100;
    pub const FLAG_RESERVED3: u32 = 0b0000_0000_0000_1000;
    pub const FLAG_AUX_CARRY: u32 = 0b0000_0000_0001_0000;
    pub const FLAG_RESERVED5: u32 = 0b0000_0000_0010_0000;
    pub const FLAG_ZERO: u32 = 0b0000_0000_0100_0000;
    pub const FLAG_SIGN: u32 = 0b0000_0000_1000_0000;
    pub const FLAG_TRAP: u32 = 0b0000_0001_0000_0000;
    pub const FLAG_INT_ENABLE: u32 = 0b0000_0010_0000_0000;
    pub const FLAG_DIRECTION: u32 = 0b0000_0100_0000_0000;
    pub const FLAG_OVERFLOW: u32 = 0b0000_1000_0000_0000;
    pub const FLAG_F15: u32 = 0b1000_0000_0000_0000; // Reserved bit 15
    pub const FLAG_MODE: u32 = 0b1000_0000_0000_0000;
    pub const FLAG_NT: u32 = 0b0100_0000_0000_0000; // Nested Task
    pub const FLAG_IOPL0: u32 = 0b0001_0000_0000_0000; // Nested Task
    pub const FLAG_IOPL1: u32 = 0b0010_0000_0000_0000; // Nested Task

    /// Calculate the code address based on CS descriptor and EIP
    pub fn calculate_code_address(&self) -> u32 {
        self.cs_desc.address + self.eip
    }
}

impl TryFrom<&[u8]> for RemoteCpuRegistersV3A {
    type Error = &'static str;
    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        parse_v3a(&buf)
    }
}

impl From<[u8; 204]> for RemoteCpuRegistersV3A {
    fn from(buf: [u8; 204]) -> RemoteCpuRegistersV3A {
        parse_v3a(&buf).unwrap()
    }
}

#[rustfmt::skip]
fn parse_v3a(buf: &[u8]) -> Result<RemoteCpuRegistersV3A, &'static str> {
    if buf.len() < 204 {
        return Err("Buffer too small for RemoteCpuRegistersV3");
    }

    let mut new_regs = RemoteCpuRegistersV3A::default();
    let mut cursor = std::io::Cursor::new(buf);

    new_regs.cr0 = cursor.read_le().unwrap();       // +00
    new_regs.eflags = cursor.read_le().unwrap();    // +04
    new_regs.eip = cursor.read_le().unwrap();       // +08
    new_regs.edi = cursor.read_le().unwrap();       // +0C
    new_regs.esi = cursor.read_le().unwrap();       // +10
    new_regs.ebp = cursor.read_le().unwrap();       // +14
    new_regs.esp = cursor.read_le().unwrap();       // +18
    new_regs.ebx = cursor.read_le().unwrap();       // +1C
    new_regs.edx = cursor.read_le().unwrap();       // +20
    new_regs.ecx = cursor.read_le().unwrap();       // +24
    new_regs.eax = cursor.read_le().unwrap();       // +28

    new_regs.dr6 = cursor.read_le().unwrap();       // +2C
    new_regs.dr7 = cursor.read_le().unwrap();       // +30

    new_regs.tr         = cursor.read_le().unwrap();
    new_regs.tr_pad     = cursor.read_le().unwrap();
    new_regs.ldt        = cursor.read_le().unwrap();
    new_regs.ldt_pad    = cursor.read_le().unwrap();
    new_regs.gs         = cursor.read_le().unwrap();
    new_regs.gs_pad     = cursor.read_le().unwrap();
    new_regs.fs         = cursor.read_le().unwrap();
    new_regs.fs_pad     = cursor.read_le().unwrap();
    new_regs.ds         = cursor.read_le().unwrap();
    new_regs.ds_pad     = cursor.read_le().unwrap();
    new_regs.ss         = cursor.read_le().unwrap();
    new_regs.ss_pad     = cursor.read_le().unwrap();
    new_regs.cs         = cursor.read_le().unwrap();
    new_regs.cs_pad     = cursor.read_le().unwrap();
    new_regs.es         = cursor.read_le().unwrap();
    new_regs.es_pad     = cursor.read_le().unwrap();

    let idx = cursor.position();
    let desc_slice = &cursor.into_inner()[idx as usize..idx as usize + 120];

    new_regs.tss_desc = read_descriptor_v2(desc_slice, 0);
    new_regs.idt_desc = read_descriptor_v2(desc_slice, 1);
    new_regs.gdt_desc = read_descriptor_v2(desc_slice, 2);
    new_regs.ldt_desc = read_descriptor_v2(desc_slice, 3);
    new_regs.gs_desc = read_descriptor_v2(desc_slice, 4);
    new_regs.fs_desc = read_descriptor_v2(desc_slice, 5);
    new_regs.ds_desc = read_descriptor_v2(desc_slice, 6);
    new_regs.ss_desc = read_descriptor_v2(desc_slice, 7);
    new_regs.cs_desc = read_descriptor_v2(desc_slice, 8);
    new_regs.es_desc = read_descriptor_v2(desc_slice, 9);
    Ok(new_regs)
}

/// [RemoteCpuRegistersV3] is the LOADALL structure for the Intel 386.
/// This structure is loaded via the LOADALL instruction, 0F 05.
#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct RemoteCpuRegistersV3B {
    pub cr0: u32,
    pub cr3: u32,
    pub eflags: u32,
    pub eip: u32,
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub dr6: u32,
    pub dr7: u32,
    pub tr: u16,
    pub tr_pad: u16,
    pub ldt: u16,
    pub ldt_pad: u16,
    pub gs: u16,
    pub gs_pad: u16,
    pub fs: u16,
    pub fs_pad: u16,
    pub ds: u16,
    pub ds_pad: u16,
    pub ss: u16,
    pub ss_pad: u16,
    pub cs: u16,
    pub cs_pad: u16,
    pub es: u16,
    pub es_pad: u16,
    pub tss_desc: SegmentDescriptorV2,
    pub idt_desc: SegmentDescriptorV2,
    pub gdt_desc: SegmentDescriptorV2,
    pub ldt_desc: SegmentDescriptorV2,
    pub gs_desc: SegmentDescriptorV2,
    pub fs_desc: SegmentDescriptorV2,
    pub ds_desc: SegmentDescriptorV2,
    pub ss_desc: SegmentDescriptorV2,
    pub cs_desc: SegmentDescriptorV2,
    pub es_desc: SegmentDescriptorV2,
}

impl Default for RemoteCpuRegistersV3B {
    fn default() -> Self {
        RemoteCpuRegistersV3B {
            cr0: 0,
            cr3: 0,
            eflags: 0x00000002, // reserved bit 1 set
            eip: 0x0000_0100,
            edi: 0,
            esi: 0,
            ebp: 0,
            esp: 0,
            ebx: 0,
            edx: 0,
            ecx: 0,
            eax: 0,
            dr6: 0,
            dr7: 0,
            tr: 0,
            tr_pad: 0,
            ldt: 0,
            ldt_pad: 0,
            gs: 0,
            gs_pad: 0,
            fs: 0,
            fs_pad: 0,
            ds: 0,
            ds_pad: 0,
            ss: 0,
            ss_pad: 0,
            cs: RemoteCpuRegistersV3B::DEFAULT_CS,
            cs_pad: 0,
            es: 0,
            es_pad: 0,

            // Default access values provided by Robert Collins
            // https://www.rcollins.org/ftp/source/386load/386load.asm
            tss_desc: SegmentDescriptorV2::default()
                .with_access(0x00008900)
                .with_limit(0xFFFFFFFF),
            idt_desc: SegmentDescriptorV2::default()
                .with_access(0x00008000)
                .with_limit(0xFFFFFFFF),
            gdt_desc: SegmentDescriptorV2::default()
                .with_access(0x00008000)
                .with_limit(0xFFFFFFFF),
            ldt_desc: SegmentDescriptorV2::default()
                .with_access(0x00008200)
                .with_limit(0xFFFFFFFF),
            gs_desc:  SegmentDescriptorV2::default()
                .with_access(0x00009300)
                .with_limit(0xFFFF),
            fs_desc:  SegmentDescriptorV2::default()
                .with_access(0x00009300)
                .with_limit(0xFFFF),
            ds_desc:  SegmentDescriptorV2::default()
                .with_access(0x00009300)
                .with_limit(0xFFFF),
            ss_desc:  SegmentDescriptorV2::default()
                .with_access(0x00009300)
                .with_limit(0xFFFF),
            cs_desc:  SegmentDescriptorV2::default()
                .with_base_address((RemoteCpuRegistersV3B::DEFAULT_CS as u32) << 4)
                .with_access(0x00009B00)
                .with_limit(0xFFFF),
            es_desc:  SegmentDescriptorV2::default()
                .with_access(0x00009300)
                .with_limit(0xFFFF),
        }
    }
}

impl RemoteCpuRegistersV3B {
    pub const DEFAULT_CS: u16 = 0x1000;

    /// Calculate the code address based on CS descriptor and EIP
    pub fn calculate_code_address(&self) -> u32 {
        self.cs_desc.address + self.eip
    }
}

impl TryFrom<&[u8]> for RemoteCpuRegistersV3B {
    type Error = &'static str;
    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        parse_v3b(&buf)
    }
}

impl From<[u8; 208]> for RemoteCpuRegistersV3B {
    fn from(buf: [u8; 208]) -> RemoteCpuRegistersV3B {
        parse_v3b(&buf).unwrap()
    }
}

#[rustfmt::skip]
fn parse_v3b(buf: &[u8]) -> Result<RemoteCpuRegistersV3B, &'static str> {
    if buf.len() < 208 {
        return Err("Buffer too small for RemoteCpuRegistersV3B");
    }

    let mut new_regs = RemoteCpuRegistersV3B::default();
    let mut cursor = std::io::Cursor::new(buf);

    new_regs.cr0 = cursor.read_le().unwrap();
    new_regs.cr3 = cursor.read_le().unwrap();
    new_regs.eflags = cursor.read_le().unwrap();
    new_regs.eip = cursor.read_le().unwrap();
    new_regs.edi = cursor.read_le().unwrap();
    new_regs.esi = cursor.read_le().unwrap();
    new_regs.ebp = cursor.read_le().unwrap();
    new_regs.esp = cursor.read_le().unwrap();
    new_regs.ebx = cursor.read_le().unwrap();
    new_regs.edx = cursor.read_le().unwrap();
    new_regs.ecx = cursor.read_le().unwrap();
    new_regs.eax = cursor.read_le().unwrap();

    new_regs.dr6 = cursor.read_le().unwrap();
    new_regs.dr7 = cursor.read_le().unwrap();

    new_regs.tr         = cursor.read_le().unwrap();
    new_regs.tr_pad     = cursor.read_le().unwrap();
    new_regs.ldt        = cursor.read_le().unwrap();
    new_regs.ldt_pad    = cursor.read_le().unwrap();
    new_regs.gs         = cursor.read_le().unwrap();
    new_regs.gs_pad     = cursor.read_le().unwrap();
    new_regs.fs         = cursor.read_le().unwrap();
    new_regs.fs_pad     = cursor.read_le().unwrap();
    new_regs.ds         = cursor.read_le().unwrap();
    new_regs.ds_pad     = cursor.read_le().unwrap();
    new_regs.ss         = cursor.read_le().unwrap();
    new_regs.ss_pad     = cursor.read_le().unwrap();
    new_regs.cs         = cursor.read_le().unwrap();
    new_regs.cs_pad     = cursor.read_le().unwrap();
    new_regs.es         = cursor.read_le().unwrap();
    new_regs.es_pad     = cursor.read_le().unwrap();

    let idx = cursor.position();
    let desc_slice = &cursor.into_inner()[idx as usize..idx as usize + 120];

    new_regs.tss_desc = read_descriptor_v2(desc_slice, 0);
    new_regs.idt_desc = read_descriptor_v2(desc_slice, 1);
    new_regs.gdt_desc = read_descriptor_v2(desc_slice, 2);
    new_regs.ldt_desc = read_descriptor_v2(desc_slice, 3);
    new_regs.gs_desc = read_descriptor_v2(desc_slice, 4);
    new_regs.fs_desc = read_descriptor_v2(desc_slice, 5);
    new_regs.ds_desc = read_descriptor_v2(desc_slice, 6);
    new_regs.ss_desc = read_descriptor_v2(desc_slice, 7);
    new_regs.cs_desc = read_descriptor_v2(desc_slice, 8);
    new_regs.es_desc = read_descriptor_v2(desc_slice, 9);
    Ok(new_regs)
}

impl_registers32!(RemoteCpuRegistersV3A);
impl_registers32!(RemoteCpuRegistersV3B);
