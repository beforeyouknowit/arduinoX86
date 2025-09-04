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
use std::io::{Seek, Write};

use crate::registers_common::RandomizeOpts;
use binrw::{binrw, BinReaderExt, BinResult, BinWrite};

#[cfg(feature = "use_moo")]
use moo::{
    prelude::MooRegisters32Init,
    types::{MooRegisters16, MooRegisters32},
};
use rand::Rng;
use rand_distr::{Beta, Distribution};

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

            fn normalize_descriptors(&mut self) {
                self.gs_desc.address = (self.gs as u32) << 4;
                self.fs_desc.address = (self.fs as u32) << 4;
                self.ds_desc.address = (self.ds as u32) << 4;
                self.ss_desc.address = (self.ss as u32) << 4;
                self.cs_desc.address = (self.cs as u32) << 4;
                self.es_desc.address = (self.es as u32) << 4;
            }
        }
    };
}

#[derive(Clone, Debug)]
pub enum RemoteCpuRegistersV3 {
    A(RemoteCpuRegistersV3A),
    B(RemoteCpuRegistersV3B),
}

// value-returning getter
macro_rules! enum_get {
    ($method:ident -> $ret:ty, $trait_m:ident) => {
        fn $method(&self) -> $ret {
            //use $Trait as _; // bring trait into scope without a name
            match self {
                Self::A(r) => r.$trait_m(),
                Self::B(r) => r.$trait_m(),
            }
        }
    };
}

// &mut-returning getter
macro_rules! enum_get_mut {
    ($method:ident -> &mut $ret:ty, $trait_m:ident) => {
        fn $method(&mut self) -> &mut $ret {
            match self {
                Self::A(r) => r.$trait_m(),
                Self::B(r) => r.$trait_m(),
            }
        }
    };
}

// setter
macro_rules! enum_set {
    ($method:ident($arg_ty:ty) => $trait_m:ident) => {
        fn $method(&mut self, value: $arg_ty) {
            match self {
                Self::A(r) => r.$trait_m(value),
                Self::B(r) => r.$trait_m(value),
            }
        }
    };
}

impl Registers32 for RemoteCpuRegistersV3 {
    // control/debug
    enum_get!(cr0 -> u32, cr0);
    //enum_get!(cr3 -> u32, cr3);
    enum_get!(dr6 -> u32, dr6);
    enum_get!(dr7 -> u32, dr7);

    enum_get_mut!(cr0_mut -> &mut u32, cr0_mut);
    //enum_get_mut!(cr3_mut -> &mut u32, cr3_mut);
    enum_get_mut!(dr6_mut -> &mut u32, dr6_mut);
    enum_get_mut!(dr7_mut -> &mut u32, dr7_mut);

    enum_set!(set_cr0(u32) => set_cr0);
    //enum_set!(set_cr3(u32) => set_cr3);
    enum_set!(set_dr6(u32) => set_dr6);
    enum_set!(set_dr7(u32) => set_dr7);

    // gprs / ip / flags
    enum_get!(eax -> u32, eax);
    enum_get!(ebx -> u32, ebx);
    enum_get!(ecx -> u32, ecx);
    enum_get!(edx -> u32, edx);
    enum_get!(esp -> u32, esp);
    enum_get!(ebp -> u32, ebp);
    enum_get!(esi -> u32, esi);
    enum_get!(edi -> u32, edi);
    enum_get!(eip -> u32, eip);
    enum_get!(eflags -> u32, eflags);

    enum_get_mut!(eax_mut -> &mut u32, eax_mut);
    enum_get_mut!(ebx_mut -> &mut u32, ebx_mut);
    enum_get_mut!(ecx_mut -> &mut u32, ecx_mut);
    enum_get_mut!(edx_mut -> &mut u32, edx_mut);
    enum_get_mut!(esp_mut -> &mut u32, esp_mut);
    enum_get_mut!(ebp_mut -> &mut u32, ebp_mut);
    enum_get_mut!(esi_mut -> &mut u32, esi_mut);
    enum_get_mut!(edi_mut -> &mut u32, edi_mut);
    enum_get_mut!(eip_mut -> &mut u32, eip_mut);
    enum_get_mut!(eflags_mut -> &mut u32, eflags_mut);

    enum_set!(set_eax(u32) => set_eax);
    enum_set!(set_ebx(u32) => set_ebx);
    enum_set!(set_ecx(u32) => set_ecx);
    enum_set!(set_edx(u32) => set_edx);
    enum_set!(set_esp(u32) => set_esp);
    enum_set!(set_ebp(u32) => set_ebp);
    enum_set!(set_esi(u32) => set_esi);
    enum_set!(set_edi(u32) => set_edi);
    enum_set!(set_eip(u32) => set_eip);
    enum_set!(set_eflags(u32) => set_eflags);

    // segments
    enum_get!(cs -> u16, cs);
    enum_get!(ds -> u16, ds);
    enum_get!(es -> u16, es);
    enum_get!(fs -> u16, fs);
    enum_get!(gs -> u16, gs);
    enum_get!(ss -> u16, ss);

    enum_get_mut!(cs_mut -> &mut u16, cs_mut);
    enum_get_mut!(ds_mut -> &mut u16, ds_mut);
    enum_get_mut!(es_mut -> &mut u16, es_mut);
    enum_get_mut!(fs_mut -> &mut u16, fs_mut);
    enum_get_mut!(gs_mut -> &mut u16, gs_mut);
    enum_get_mut!(ss_mut -> &mut u16, ss_mut);

    enum_set!(set_cs(u16) => set_cs);
    enum_set!(set_ds(u16) => set_ds);
    enum_set!(set_es(u16) => set_es);
    enum_set!(set_fs(u16) => set_fs);
    enum_set!(set_gs(u16) => set_gs);
    enum_set!(set_ss(u16) => set_ss);

    fn normalize_descriptors(&mut self) {
        match self {
            RemoteCpuRegistersV3::A(regs) => regs.normalize_descriptors(),
            RemoteCpuRegistersV3::B(regs) => regs.normalize_descriptors(),
        }
    }
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
#[derive(Copy, Clone, Debug, Default)]
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

    pub fn base_address(&self) -> u32 {
        self.address
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
#[derive(Clone, Debug)]
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
            cr0: 0x7FFE_FFF0,
            eflags: 0xFFFC_0002, // reserved bit 1 set
            eip: 0x0000_0100,
            edi: 0,
            esi: 0,
            ebp: 0,
            esp: 0,
            ebx: 0,
            edx: 0,
            ecx: 0,
            eax: 0,
            dr6: 0xFFFF_0FF0,
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
                .with_access(0x00009300)
                .with_limit(0xFFFF),
            es_desc:  SegmentDescriptorV2::default()
                .with_access(0x00009300)
                .with_limit(0xFFFF),
        }
    }
}

impl RemoteCpuRegistersV3A {
    pub const DEFAULT_CS: u16 = 0x1000;

    pub const FLAGS_RESERVED_CR0: u32 = 0x7FFE_FFF0; // Reserved bits in CR0
    pub const FLAGS_RESERVED_SET: u32 = 0xFFFC_0002; // Reserved bit 1 set
    pub const FLAGS_RESERVED_MASK: u32 = 0xFFFF_7FD7;
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

    pub fn to_buffer<WS: Write + Seek>(&self, buffer: &mut WS) -> BinResult<()> {
        self.write_le(buffer)
    }

    pub fn clear_trap_flag(&mut self) {
        // Clear the trap flag (bit 8) in the flags register.
        self.eflags &= !0x0100; // Clear bit 8
    }
    pub fn clear_interrupt_flag(&mut self) {
        // Clear the interrupt flag (bit 9) in the flags register.
        self.eflags &= !0x0200; // Clear bit 9
    }

    pub fn weighted_u16(
        weight_zero: f32,
        weight_ones: f32,
        rand: &mut rand::rngs::StdRng,
        register_beta: &mut Beta<f64>,
    ) -> u16 {
        let random_value: f32 = rand.random();
        if random_value < weight_zero {
            0
        }
        else if random_value < weight_zero + weight_ones {
            0xFFFF // All bits set to 1
        }
        else {
            let value: u16 = (register_beta.sample(rand) * u16::MAX as f64) as u16;
            value
        }
    }

    pub fn weighted_u32(
        weight_zero: f32,
        weight_ones: f32,
        rand: &mut rand::rngs::StdRng,
        register_beta: &mut Beta<f64>,
    ) -> u32 {
        let random_value: f32 = rand.random();
        if random_value < weight_zero {
            0
        }
        else if random_value < weight_zero + weight_ones {
            0xFFFF // All bits set to 1
        }
        else {
            let value: u32 = (register_beta.sample(rand) * u32::MAX as f64) as u32;
            value
        }
    }

    #[rustfmt::skip]
    pub fn randomize(&mut self, opts: &RandomizeOpts, rand: &mut rand::rngs::StdRng, beta: &mut Beta<f64>) {
        *self = RemoteCpuRegistersV3A::default(); // Reset all registers to default values

        if opts.randomize_flags {
            self.eflags = (rand.random::<u32>() | RemoteCpuRegistersV3A::FLAGS_RESERVED_SET) & RemoteCpuRegistersV3A::FLAGS_RESERVED_MASK; // Set reserved bit
        }
        if opts.clear_trap_flag {
            self.clear_trap_flag();
        }
        if opts.clear_interrupt_flag {
            self.clear_interrupt_flag();
        }

        if opts.randomize_general {
            self.eax = RemoteCpuRegistersV3A::weighted_u32(opts.weight_zero, opts.weight_ones, rand, beta);
            self.ebx = RemoteCpuRegistersV3A::weighted_u32(opts.weight_zero, opts.weight_ones, rand, beta);
            self.ecx = RemoteCpuRegistersV3A::weighted_u32(opts.weight_zero, opts.weight_ones, rand, beta);
            self.edx = RemoteCpuRegistersV3A::weighted_u32(opts.weight_zero, opts.weight_ones, rand, beta);
            self.esp = RemoteCpuRegistersV3A::weighted_u32(opts.weight_zero, opts.weight_ones, rand, beta);
            self.ebp = RemoteCpuRegistersV3A::weighted_u32(opts.weight_zero, opts.weight_ones, rand, beta);
            self.esi = RemoteCpuRegistersV3A::weighted_u32(opts.weight_zero, opts.weight_ones, rand, beta);
            self.edi = RemoteCpuRegistersV3A::weighted_u32(opts.weight_zero, opts.weight_ones, rand, beta);
            self.ds = RemoteCpuRegistersV3A::weighted_u16(opts.weight_zero, opts.weight_ones, rand, beta);
            self.ss = RemoteCpuRegistersV3A::weighted_u16(opts.weight_zero, opts.weight_ones, rand, beta);
            self.es = RemoteCpuRegistersV3A::weighted_u16(opts.weight_zero, opts.weight_ones, rand, beta);
            self.fs = RemoteCpuRegistersV3A::weighted_u16(opts.weight_zero, opts.weight_ones, rand, beta);
            self.gs = RemoteCpuRegistersV3A::weighted_u16(opts.weight_zero, opts.weight_ones, rand, beta);
            self.cs = RemoteCpuRegistersV3A::weighted_u16(opts.weight_zero, opts.weight_ones, rand, beta);
        }

        if opts.randomize_ip {
            self.eip = rand.random::<u32>() & opts.eip_mask;
        }

        // Set SP to even value.
        self.esp = self.esp & !1;
        // Use the sp_odd_chance to set it to an odd value based on the configured percentage.
        let odd_chance = rand.random::<f32>();
        if odd_chance < opts.weight_sp_odd {
            self.esp |= 1; // Set the least significant bit to 1 to make it odd
        }

        // Set sp to minimum value if beneath.
        if self.esp < opts.sp_min_value32 {
            self.esp = opts.sp_min_value32;
        }

        // Set sp to maximum value if above.
        if self.esp > opts.sp_max_value32 {
            self.esp = opts.sp_max_value32;
        }

        if opts.randomize_msw {
            self.cr0 = rand.random::<u32>() & !RemoteCpuRegistersV3A::FLAGS_RESERVED_CR0; // Keep reserved bits
        }

        if opts.randomize_tr {
            self.tr = rand.random::<u16>();
        }

        if opts.randomize_ldt {
            self.ldt = rand.random::<u16>();
        }

        if opts.randomize_segment_descriptors {
            let mut base_address: u32 = (rand.random::<u32>()) << 4;
            let limit: u32 = 0x0000_FFFF;

            // Randomize segment descriptors
            self.es_desc = SegmentDescriptorV2::default()
                .with_base_address(base_address)
                .with_limit(limit);

            base_address = (rand.random::<u32>()) << 4;
            self.cs_desc = SegmentDescriptorV2::default()
                .with_base_address(base_address)
                .with_limit(limit);

            base_address = (rand.random::<u32>()) << 4;
            self.ss_desc = SegmentDescriptorV2::default()
                .with_base_address(base_address)
                .with_limit(limit);

            base_address = (rand.random::<u32>()) << 4;
            self.ds_desc = SegmentDescriptorV2::default()
                .with_base_address(base_address)
                .with_limit(limit);

            base_address = (rand.random::<u32>()) << 4;
            self.es_desc = SegmentDescriptorV2::default()
                .with_base_address(base_address)
                .with_limit(limit);

            base_address = (rand.random::<u32>()) << 4;
            self.fs_desc = SegmentDescriptorV2::default()
                .with_base_address(base_address)
                .with_limit(limit);
        }
    }

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
#[derive(Clone, Debug)]
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
                .with_access(0x00009300)
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

impl From<&RemoteCpuRegistersV3A> for RemoteCpuRegistersV3B {
    fn from(regs: &RemoteCpuRegistersV3A) -> RemoteCpuRegistersV3B {
        RemoteCpuRegistersV3B {
            cr0: regs.cr0,
            cr3: 0,
            eflags: regs.eflags,
            eip: regs.eip,
            edi: regs.edi,
            esi: regs.esi,
            ebp: regs.ebp,
            esp: regs.esp,
            ebx: regs.ebx,
            edx: regs.edx,
            ecx: regs.ecx,
            eax: regs.eax,
            dr6: regs.dr6,
            dr7: regs.dr7,
            tr: regs.tr,
            tr_pad: regs.tr_pad,
            ldt: regs.ldt,
            ldt_pad: regs.ldt_pad,
            gs: regs.gs,
            gs_pad: regs.gs_pad,
            fs: regs.fs,
            fs_pad: regs.fs_pad,
            ds: regs.ds,
            ds_pad: regs.ds_pad,
            ss: regs.ss,
            ss_pad: regs.ss_pad,
            cs: regs.cs,
            cs_pad: regs.cs_pad,
            es: regs.es,
            es_pad: regs.es_pad,

            tss_desc: regs.tss_desc,
            idt_desc: regs.idt_desc,
            gdt_desc: regs.gdt_desc,
            ldt_desc: regs.ldt_desc,
            gs_desc:  regs.gs_desc,
            fs_desc:  regs.fs_desc,
            ds_desc:  regs.ds_desc,
            ss_desc:  regs.ss_desc,
            cs_desc:  regs.cs_desc,
            es_desc:  regs.es_desc,
        }
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

#[cfg(feature = "use_moo")]
impl From<RemoteCpuRegistersV3A> for MooRegisters32 {
    fn from(regs: RemoteCpuRegistersV3A) -> MooRegisters32 {
        MooRegisters32::from(&regs)
    }
}

#[cfg(feature = "use_moo")]
impl From<&RemoteCpuRegistersV3A> for MooRegisters32 {
    fn from(regs: &RemoteCpuRegistersV3A) -> MooRegisters32 {
        (&(MooRegisters32Init {
            cr0: regs.cr0,
            cr3: 0,
            eax: regs.eax,
            ebx: regs.ebx,
            ecx: regs.ecx,
            edx: regs.edx,
            esp: regs.esp,
            ebp: regs.ebp,
            esi: regs.esi,
            edi: regs.edi,
            eip: regs.eip,
            dr6: regs.dr6,
            dr7: regs.dr7,
            eflags: regs.eflags,
            cs: regs.cs as u32,
            ds: regs.ds as u32,
            es: regs.es as u32,
            fs: regs.fs as u32,
            gs: regs.gs as u32,
            ss: regs.ss as u32,
        }))
            .into()
    }
}

#[cfg(feature = "use_moo")]
impl From<RemoteCpuRegistersV3B> for MooRegisters32 {
    fn from(regs: RemoteCpuRegistersV3B) -> MooRegisters32 {
        MooRegisters32::from(&regs)
    }
}

#[cfg(feature = "use_moo")]
impl From<&RemoteCpuRegistersV3B> for MooRegisters32 {
    fn from(regs: &RemoteCpuRegistersV3B) -> MooRegisters32 {
        (&(MooRegisters32Init {
            cr0: regs.cr0,
            cr3: regs.cr3,
            eax: regs.eax,
            ebx: regs.ebx,
            ecx: regs.ecx,
            edx: regs.edx,
            esp: regs.esp,
            ebp: regs.ebp,
            esi: regs.esi,
            edi: regs.edi,
            eip: regs.eip,
            dr6: regs.dr6,
            dr7: regs.dr7,
            eflags: regs.eflags,
            cs: regs.cs as u32,
            ds: regs.ds as u32,
            es: regs.es as u32,
            fs: regs.fs as u32,
            gs: regs.gs as u32,
            ss: regs.ss as u32,
        }))
            .into()
    }
}
