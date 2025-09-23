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
#![allow(dead_code)]

use crate::{trace_banner, trace_log, TestContext};
use arduinox86_client::Registers32;
use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Formatter},
};

use moo::types::effective_address::MooEffectiveAddress;

#[derive(Copy, Clone, Debug)]
pub enum Displacement {
    NoDisp,
    Pending8,
    Pending16,
    Pending32,
    Disp8(i8),
    Disp16(i16),
    Disp32(i32),
}

impl Display for Displacement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Displacement::NoDisp => write!(f, "[None]"),
            Displacement::Pending8 => write!(f, "PENDING8"),
            Displacement::Pending16 => write!(f, "PENDING16"),
            Displacement::Pending32 => write!(f, "PENDING32"),
            Displacement::Disp8(v) => {
                if *v < 0 {
                    write!(f, "-{:X}h", -(*v as i16))
                }
                else {
                    write!(f, "+{:X}h", v)
                }
            }
            Displacement::Disp16(v) => {
                if *v < 0 {
                    write!(f, "-{:X}h", -(*v as i32))
                }
                else {
                    write!(f, "+{:X}h", v)
                }
            }
            Displacement::Disp32(v) => {
                if *v < 0 {
                    write!(f, "-{:X}h", -(*v as i64))
                }
                else {
                    write!(f, "+{:X}h", v)
                }
            }
        }
    }
}

impl From<Displacement> for i8 {
    fn from(value: Displacement) -> Self {
        match value {
            Displacement::Disp8(v) => v,
            _ => 0,
        }
    }
}

impl From<Displacement> for i16 {
    fn from(value: Displacement) -> Self {
        match value {
            Displacement::Disp16(v) => v,
            _ => 0,
        }
    }
}

impl From<Displacement> for i32 {
    fn from(value: Displacement) -> Self {
        match value {
            Displacement::Disp32(v) => v,
            _ => 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SibScale {
    One,
    Two,
    Four,
    Eight,
}

impl Display for SibScale {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SibScale::One => write!(f, "*1"),
            SibScale::Two => write!(f, "*2"),
            SibScale::Four => write!(f, "*4"),
            SibScale::Eight => write!(f, "*8"),
        }
    }
}

impl From<&SibScale> for u32 {
    fn from(value: &SibScale) -> u32 {
        match value {
            SibScale::One => 1,
            SibScale::Two => 2,
            SibScale::Four => 4,
            SibScale::Eight => 8,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AddressingMode {
    Sixteen(AddressingMode16),
    ThirtyTwo(AddressingMode32),
}

impl Display for AddressingMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingMode::Sixteen(mode) => write!(f, "{}", mode),
            AddressingMode::ThirtyTwo(mode) => write!(f, "{}", mode),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AddressingMode16 {
    RegisterMode,
    Address { base: Register16, offset: AddressOffset16 },
}

impl AddressingMode16 {
    pub fn is_register_mode(&self) -> bool {
        matches!(self, AddressingMode16::RegisterMode)
    }

    pub fn is_address_mode(&self) -> bool {
        matches!(self, AddressingMode16::Address { .. })
    }
}

impl Display for AddressingMode16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingMode16::RegisterMode => write!(f, "[Reg]"),
            AddressingMode16::Address { base, offset } => {
                write!(f, "[{}:{}]", base, offset)
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AddressingMode32 {
    RegisterMode,
    Address { base: Register32, offset: AddressOffset32 },
}

impl AddressingMode32 {
    pub fn is_register_mode(&self) -> bool {
        matches!(self, AddressingMode32::RegisterMode)
    }

    pub fn is_address_mode(&self) -> bool {
        matches!(self, AddressingMode32::Address { .. })
    }

    pub fn is_sib(&self) -> bool {
        match self {
            AddressingMode32::RegisterMode => false,
            AddressingMode32::Address { base: _, offset } => offset.is_sib(),
        }
    }
}

impl Display for AddressingMode32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingMode32::RegisterMode => write!(f, "[Reg]"),
            AddressingMode32::Address { base, offset } => {
                write!(f, "[{}:{}]", base, offset)
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AddressOffset {
    Sixteen(AddressOffset16),
    ThirtyTwo(AddressOffset32),
}

#[derive(Copy, Clone, Debug)]
pub enum AddressOffset16 {
    None,
    BxSi,
    BxDi,
    BpSi,
    BpDi,
    Si,
    Di,
    Disp16(i16),
    Bx,
    BxSiDisp8(i8),
    BxDiDisp8(i8),
    BpSiDisp8(i8),
    BpDiDisp8(i8),
    SiDisp8(i8),
    DiDisp8(i8),
    BpDisp8(i8),
    BxDisp8(i8),
    BxSiDisp16(i16),
    BxDiDisp16(i16),
    BpSiDisp16(i16),
    BpDiDisp16(i16),
    SiDisp16(i16),
    DiDisp16(i16),
    BpDisp16(i16),
    BxDisp16(i16),
}

impl AddressOffset16 {
    #[rustfmt::skip]
    pub fn calculate(&self, regs: &dyn Registers32) -> u32 {
        use AddressOffset16::*;
        let offset = match self {
            None => 0,
            BxSi => (regs.ebx() & 0xFFFF).wrapping_add(regs.esi() & 0xFFFF),
            BxDi => (regs.ebx() & 0xFFFF).wrapping_add(regs.edi() & 0xFFFF),
            BpSi => (regs.ebp() & 0xFFFF).wrapping_add(regs.esi() & 0xFFFF),
            BpDi => (regs.ebp() & 0xFFFF).wrapping_add(regs.edi() & 0xFFFF),
            Si => regs.esi() & 0xFFFF,
            Di => regs.edi() & 0xFFFF,
            Disp16(disp) => *disp as u32,
            Bx => regs.ebx() & 0xFFFF,
            BxSiDisp8(disp) => (regs.ebx() & 0xFFFF).wrapping_add(regs.esi() & 0xFFFF).wrapping_add(*disp as u32),
            BxDiDisp8(disp) => (regs.ebx() & 0xFFFF).wrapping_add(regs.edi() & 0xFFFF).wrapping_add(*disp as u32),
            BpSiDisp8(disp) => (regs.ebp() & 0xFFFF).wrapping_add(regs.esi() & 0xFFFF).wrapping_add(*disp as u32),
            BpDiDisp8(disp) => (regs.ebp() & 0xFFFF).wrapping_add(regs.edi() & 0xFFFF).wrapping_add(*disp as u32),
            SiDisp8(disp) => (regs.esi() & 0xFFFF).wrapping_add(*disp as u32),
            DiDisp8(disp) => (regs.edi() & 0xFFFF).wrapping_add(*disp as u32),
            BpDisp8(disp) => (regs.ebp() & 0xFFFF).wrapping_add(*disp as u32),
            BxDisp8(disp) => (regs.ebx() & 0xFFFF).wrapping_add(*disp as u32),
            BxSiDisp16(disp) => (regs.ebx() & 0xFFFF).wrapping_add(regs.esi() & 0xFFFF).wrapping_add(*disp as u32),
            BxDiDisp16(disp) => (regs.ebx() & 0xFFFF).wrapping_add(regs.edi() & 0xFFFF).wrapping_add(*disp as u32),
            BpSiDisp16(disp) => (regs.ebp() & 0xFFFF).wrapping_add(regs.esi() & 0xFFFF).wrapping_add(*disp as u32),
            BpDiDisp16(disp) => (regs.ebp() & 0xFFFF).wrapping_add(regs.edi() & 0xFFFF).wrapping_add(*disp as u32),
            SiDisp16(disp) => (regs.esi() & 0xFFFF).wrapping_add(*disp as u32),
            DiDisp16(disp) => (regs.edi() & 0xFFFF).wrapping_add(*disp as u32),
            BpDisp16(disp) => (regs.ebp() & 0xFFFF).wrapping_add(*disp as u32),
            BxDisp16(disp) => (regs.ebx() & 0xFFFF).wrapping_add(*disp as u32),
        } & 0xFFFF;

        offset
    }

    #[rustfmt::skip]
    pub fn calculate_effective_address(&self, base_segment: Register16, regs: &dyn Registers32) -> u32 {
        let offset = self.calculate(regs);

        let base_addr = match base_segment {
            Register16::ES => regs.es_base(),
            Register16::CS => regs.cs_base(),
            Register16::SS => regs.ss_base(),
            Register16::DS => regs.ds_base(),
            Register16::FS => regs.fs_base(),
            Register16::GS => regs.gs_base(),
            _ => 0,
        };

        //log::warn!("16-bit EA calc: base_seg={base_segment:?} base_addr={base_addr:08X} offset={offset:08X} => EA={:08X}", base_addr.wrapping_add(offset));
        base_addr.wrapping_add(offset)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BaseRegister {
    None,
    Some(Register32),
}

impl BaseRegister {
    pub fn is_some(&self) -> bool {
        !matches!(self, BaseRegister::None)
    }
    pub fn is_none(&self) -> bool {
        matches!(self, BaseRegister::None)
    }
}

impl Display for BaseRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseRegister::None => write!(f, ""),
            BaseRegister::Some(reg) => write!(f, "{}", reg),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ScaledIndex {
    None,
    EaxScaled(SibScale),
    EcxScaled(SibScale),
    EdxScaled(SibScale),
    EbxScaled(SibScale),
    EbpScaled(SibScale),
    EsiScaled(SibScale),
    EdiScaled(SibScale),
}

impl ScaledIndex {
    #[inline]
    pub fn is_some(&self) -> bool {
        !matches!(self, ScaledIndex::None)
    }
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, ScaledIndex::None)
    }
}

impl Display for ScaledIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScaledIndex::None => write!(f, ""),
            ScaledIndex::EaxScaled(scale) => write!(f, "eax{}", scale),
            ScaledIndex::EcxScaled(scale) => write!(f, "ecx{}", scale),
            ScaledIndex::EdxScaled(scale) => write!(f, "edx{}", scale),
            ScaledIndex::EbxScaled(scale) => write!(f, "ebx{}", scale),
            ScaledIndex::EbpScaled(scale) => write!(f, "ebp{}", scale),
            ScaledIndex::EsiScaled(scale) => write!(f, "esi{}", scale),
            ScaledIndex::EdiScaled(scale) => write!(f, "edi{}", scale),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AddressOffset32 {
    None,
    Eax,
    Ecx,
    Edx,
    Ebx,
    Disp32(i32),
    Ebp,
    Esi,
    Edi,
    EaxDisp8(i8),
    EcxDisp8(i8),
    EdxDisp8(i8),
    EbxDisp8(i8),
    EspDisp8(i8),
    EbpDisp8(i8),
    EsiDisp8(i8),
    EdiDisp8(i8),
    EaxDisp32(i32),
    EcxDisp32(i32),
    EdxDisp32(i32),
    EbxDisp32(i32),
    EspDisp32(i32),
    EbpDisp32(i32),
    EsiDisp32(i32),
    EdiDisp32(i32),
    SibPending,
    Sib(BaseRegister, ScaledIndex),
    SibDisp8(BaseRegister, ScaledIndex, i8),
    SibDisp32(BaseRegister, ScaledIndex, i32),
    SibDisp8Ebp(BaseRegister, ScaledIndex, i8),
    SibDisp32Ebp(BaseRegister, ScaledIndex, i32),
}

macro_rules! disp_hex {
    ($t:ty, $x:expr) => {{
        // Accept value or reference: coerce to &T via Borrow, then copy out (Copy for ints).
        let v: $t = *<$t as Borrow<$t>>::borrow(&$x);
        format!("{v:X}h")
    }};
}

macro_rules! signed_hex {
    ($t:ty, $x:expr) => {{
        // Accept value or reference: coerce to &T via Borrow, then copy out (Copy for ints).
        let v: $t = *<$t as Borrow<$t>>::borrow(&$x);

        let neg = v < 0;
        let mag = v.unsigned_abs() as u32; // magnitude as unsigned

        // Print in decimal if small
        if mag < 10 {
            if mag == 0 {
                format!("")
            }
            else if neg {
                format!("-{}", mag)
            }
            else {
                format!("+{}", mag)
            }
        }
        else {
            // Hex without width
            let s = format!("{:X}", mag);

            if neg {
                format!("-{s}h")
            }
            else {
                format!("+{s}h")
            }
        }
    }};
}

impl Display for AddressOffset16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use AddressOffset16::*;
        match self {
            None => write!(f, ""),
            BxSi => write!(f, "bx+si"),
            BxDi => write!(f, "bx+di"),
            BpSi => write!(f, "bp+si"),
            BpDi => write!(f, "bp+di"),
            Si => write!(f, "si"),
            Di => write!(f, "di"),
            Disp16(disp) => write!(f, "{}", disp_hex!(i16, disp)),
            Bx => write!(f, "bx"),
            BxSiDisp8(disp) => write!(f, "bx+si{}", signed_hex!(i8, disp)),
            BxDiDisp8(disp) => write!(f, "bx+di{}", signed_hex!(i8, disp)),
            BpSiDisp8(disp) => write!(f, "bp+si{}", signed_hex!(i8, disp)),
            BpDiDisp8(disp) => write!(f, "bp+di{}", signed_hex!(i8, disp)),
            SiDisp8(disp) => write!(f, "si{}", signed_hex!(i8, disp)),
            DiDisp8(disp) => write!(f, "di{}", signed_hex!(i8, disp)),
            BpDisp8(disp) => write!(f, "bp{}", signed_hex!(i8, disp)),
            BxDisp8(disp) => write!(f, "bx{}", signed_hex!(i8, disp)),
            BxSiDisp16(disp) => write!(f, "bx+si{}", signed_hex!(i16, disp)),
            BxDiDisp16(disp) => write!(f, "bx+di{}", signed_hex!(i16, disp)),
            BpSiDisp16(disp) => write!(f, "bp+si{}", signed_hex!(i16, disp)),
            BpDiDisp16(disp) => write!(f, "bp+di{}", signed_hex!(i16, disp)),
            SiDisp16(disp) => write!(f, "si{}", signed_hex!(i16, disp)),
            DiDisp16(disp) => write!(f, "di{}", signed_hex!(i16, disp)),
            BpDisp16(disp) => write!(f, "bp{}", signed_hex!(i16, disp)),
            BxDisp16(disp) => write!(f, "bx{}", signed_hex!(i16, disp)),
        }
    }
}

impl Display for AddressOffset32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use AddressOffset32::*;

        match self {
            Eax => write!(f, "eax"),
            Ecx => write!(f, "ecx"),
            Edx => write!(f, "edx"),
            Ebx => write!(f, "ebx"),
            Disp32(disp) => write!(f, "{}", disp_hex!(i32, disp)),
            Ebp => write!(f, "ebp"),
            Esi => write!(f, "esi"),
            Edi => write!(f, "edi"),
            EaxDisp8(disp) => write!(f, "eax{}", signed_hex!(i8, disp)),
            EcxDisp8(disp) => write!(f, "ecx{}", signed_hex!(i8, disp)),
            EdxDisp8(disp) => write!(f, "edx{}", signed_hex!(i8, disp)),
            EbxDisp8(disp) => write!(f, "ebx{}", signed_hex!(i8, disp)),
            EspDisp8(disp) => write!(f, "esp{}", signed_hex!(i8, disp)),
            EbpDisp8(disp) => write!(f, "ebp{}", signed_hex!(i8, disp)),
            EsiDisp8(disp) => write!(f, "esi{}", signed_hex!(i8, disp)),
            EdiDisp8(disp) => write!(f, "edi{}", signed_hex!(i8, disp)),
            EaxDisp32(disp) => write!(f, "eax{}", signed_hex!(i32, disp)),
            EcxDisp32(disp) => write!(f, "ecx{}", signed_hex!(i32, disp)),
            EdxDisp32(disp) => write!(f, "edx{}", signed_hex!(i32, disp)),
            EbxDisp32(disp) => write!(f, "ebx{}", signed_hex!(i32, disp)),
            EspDisp32(disp) => write!(f, "esp{}", signed_hex!(i32, disp)),
            EbpDisp32(disp) => write!(f, "ebp{}", signed_hex!(i32, disp)),
            EsiDisp32(disp) => write!(f, "esi{}", signed_hex!(i32, disp)),
            EdiDisp32(disp) => write!(f, "edi{}", signed_hex!(i32, disp)),
            SibPending => write!(f, "**INVALID**"),
            Sib(base, scale) => {
                let plus = if base.is_some() && scale.is_some() { "+" } else { "" };
                write!(f, "{base}{plus}{scale}")
            }
            SibDisp8(base, scale, disp) => {
                let plus = if base.is_some() && scale.is_some() { "+" } else { "" };
                write!(f, "{base}{plus}{scale}{}", signed_hex!(i8, disp))
            }
            SibDisp32(base, scale, disp) => {
                let plus = if base.is_some() && scale.is_some() { "+" } else { "" };
                let display = if base.is_none() && scale.is_none() {
                    disp_hex!(i32, disp)
                }
                else {
                    signed_hex!(i32, disp)
                };
                write!(f, "{base}{plus}{scale}{display}")
            }
            SibDisp8Ebp(base, scale, disp) => {
                let plus = if base.is_some() && scale.is_some() { "+" } else { "" };
                write!(f, "{base}{plus}{scale}{}", signed_hex!(i8, disp))
            }
            SibDisp32Ebp(base, scale, disp) => {
                let plus = if base.is_some() && scale.is_some() { "+" } else { "" };
                write!(f, "{base}{plus}{scale}{}", signed_hex!(i32, disp))
            }
            None => write!(f, ""),
        }
    }
}

impl AddressOffset32 {
    pub fn is_sib(&self) -> bool {
        matches!(
            self,
            AddressOffset32::Sib(_, _)
                | AddressOffset32::SibDisp8(_, _, _)
                | AddressOffset32::SibDisp32(_, _, _)
                | AddressOffset32::SibDisp8Ebp(_, _, _)
                | AddressOffset32::SibDisp32Ebp(_, _, _)
        )
    }

    pub fn calculate(&self, regs: &dyn Registers32) -> u32 {
        use AddressOffset32::*;
        let offset = match self {
            None => 0,
            Eax => regs.eax(),
            Ecx => regs.ecx(),
            Edx => regs.edx(),
            Ebx => regs.ebx(),
            Disp32(disp) => *disp as u32,
            Ebp => regs.ebp(),
            Esi => regs.esi(),
            Edi => regs.edi(),
            EaxDisp8(disp) => regs.eax().wrapping_add(*disp as u32),
            EcxDisp8(disp) => regs.ecx().wrapping_add(*disp as u32),
            EdxDisp8(disp) => regs.edx().wrapping_add(*disp as u32),
            EbxDisp8(disp) => regs.ebx().wrapping_add(*disp as u32),
            EspDisp8(disp) => regs.esp().wrapping_add(*disp as u32),
            EbpDisp8(disp) => regs.ebp().wrapping_add(*disp as u32),
            EsiDisp8(disp) => regs.esi().wrapping_add(*disp as u32),
            EdiDisp8(disp) => regs.edi().wrapping_add(*disp as u32),
            EaxDisp32(disp) => regs.eax().wrapping_add(*disp as u32),
            EcxDisp32(disp) => regs.ecx().wrapping_add(*disp as u32),
            EdxDisp32(disp) => regs.edx().wrapping_add(*disp as u32),
            EbxDisp32(disp) => regs.ebx().wrapping_add(*disp as u32),
            EspDisp32(disp) => regs.esp().wrapping_add(*disp as u32),
            EbpDisp32(disp) => regs.ebp().wrapping_add(*disp as u32),
            EsiDisp32(disp) => regs.esi().wrapping_add(*disp as u32),
            EdiDisp32(disp) => regs.edi().wrapping_add(*disp as u32),
            SibPending => 0, // Invalid
            Sib(base, index) => {
                let base_val = match base {
                    BaseRegister::None => 0,
                    BaseRegister::Some(reg) => match reg {
                        Register32::EAX => regs.eax(),
                        Register32::ECX => regs.ecx(),
                        Register32::EDX => regs.edx(),
                        Register32::EBX => regs.ebx(),
                        Register32::ESP => regs.esp(),
                        Register32::EBP => regs.ebp(),
                        Register32::ESI => regs.esi(),
                        Register32::EDI => regs.edi(),
                        _ => 0,
                    },
                };
                let index_val = match index {
                    ScaledIndex::None => 0,
                    ScaledIndex::EaxScaled(scale) => regs.eax().wrapping_mul(scale.into()),
                    ScaledIndex::EcxScaled(scale) => regs.ecx().wrapping_mul(scale.into()),
                    ScaledIndex::EdxScaled(scale) => regs.edx().wrapping_mul(scale.into()),
                    ScaledIndex::EbxScaled(scale) => regs.ebx().wrapping_mul(scale.into()),
                    ScaledIndex::EbpScaled(scale) => regs.ebp().wrapping_mul(scale.into()),
                    ScaledIndex::EsiScaled(scale) => regs.esi().wrapping_mul(scale.into()),
                    ScaledIndex::EdiScaled(scale) => regs.edi().wrapping_mul(scale.into()),
                };
                base_val.wrapping_add(index_val)
            }
            SibDisp8(base, index, disp) => {
                let base_val = match base {
                    BaseRegister::None => 0,
                    BaseRegister::Some(reg) => match reg {
                        Register32::EAX => regs.eax(),
                        Register32::ECX => regs.ecx(),
                        Register32::EDX => regs.edx(),
                        Register32::EBX => regs.ebx(),
                        Register32::ESP => regs.esp(),
                        Register32::EBP => regs.ebp(),
                        Register32::ESI => regs.esi(),
                        Register32::EDI => regs.edi(),
                        _ => 0,
                    },
                };
                let index_val = match index {
                    ScaledIndex::None => 0,
                    ScaledIndex::EaxScaled(scale) => regs.eax().wrapping_mul(scale.into()),
                    ScaledIndex::EcxScaled(scale) => regs.ecx().wrapping_mul(scale.into()),
                    ScaledIndex::EdxScaled(scale) => regs.edx().wrapping_mul(scale.into()),
                    ScaledIndex::EbxScaled(scale) => regs.ebx().wrapping_mul(scale.into()),
                    ScaledIndex::EbpScaled(scale) => regs.ebp().wrapping_mul(scale.into()),
                    ScaledIndex::EsiScaled(scale) => regs.esi().wrapping_mul(scale.into()),
                    ScaledIndex::EdiScaled(scale) => regs.edi().wrapping_mul(scale.into()),
                };
                base_val.wrapping_add(index_val).wrapping_add(*disp as u32)
            }
            SibDisp32(base, index, disp) => {
                let base_val = match base {
                    BaseRegister::None => 0,
                    BaseRegister::Some(reg) => match reg {
                        Register32::EAX => regs.eax(),
                        Register32::ECX => regs.ecx(),
                        Register32::EDX => regs.edx(),
                        Register32::EBX => regs.ebx(),
                        Register32::ESP => regs.esp(),
                        Register32::EBP => regs.ebp(),
                        Register32::ESI => regs.esi(),
                        Register32::EDI => regs.edi(),
                        _ => 0,
                    },
                };
                let index_val = match index {
                    ScaledIndex::None => 0,
                    ScaledIndex::EaxScaled(scale) => regs.eax().wrapping_mul(scale.into()),
                    ScaledIndex::EcxScaled(scale) => regs.ecx().wrapping_mul(scale.into()),
                    ScaledIndex::EdxScaled(scale) => regs.edx().wrapping_mul(scale.into()),
                    ScaledIndex::EbxScaled(scale) => regs.ebx().wrapping_mul(scale.into()),
                    ScaledIndex::EbpScaled(scale) => regs.ebp().wrapping_mul(scale.into()),
                    ScaledIndex::EsiScaled(scale) => regs.esi().wrapping_mul(scale.into()),
                    ScaledIndex::EdiScaled(scale) => regs.edi().wrapping_mul(scale.into()),
                };
                base_val.wrapping_add(index_val).wrapping_add(*disp as u32)
            }
            SibDisp8Ebp(base, index, disp) => {
                let base_val = match base {
                    BaseRegister::None => 0,
                    BaseRegister::Some(reg) => match reg {
                        Register32::EAX => regs.eax(),
                        Register32::ECX => regs.ecx(),
                        Register32::EDX => regs.edx(),
                        Register32::EBX => regs.ebx(),
                        Register32::ESP => regs.esp(),
                        Register32::EBP => regs.ebp(),
                        Register32::ESI => regs.esi(),
                        Register32::EDI => regs.edi(),
                        _ => 0,
                    },
                };
                let index_val = match index {
                    ScaledIndex::None => 0,
                    ScaledIndex::EaxScaled(scale) => regs.eax().wrapping_mul(scale.into()),
                    ScaledIndex::EcxScaled(scale) => regs.ecx().wrapping_mul(scale.into()),
                    ScaledIndex::EdxScaled(scale) => regs.edx().wrapping_mul(scale.into()),
                    ScaledIndex::EbxScaled(scale) => regs.ebx().wrapping_mul(scale.into()),
                    ScaledIndex::EbpScaled(scale) => regs.ebp().wrapping_mul(scale.into()),
                    ScaledIndex::EsiScaled(scale) => regs.esi().wrapping_mul(scale.into()),
                    ScaledIndex::EdiScaled(scale) => regs.edi().wrapping_mul(scale.into()),
                };
                base_val.wrapping_add(index_val).wrapping_add(*disp as u32)
            }
            SibDisp32Ebp(base, index, disp) => {
                let base_val = match base {
                    BaseRegister::None => 0,
                    BaseRegister::Some(reg) => match reg {
                        Register32::EAX => regs.eax(),
                        Register32::ECX => regs.ecx(),
                        Register32::EDX => regs.edx(),
                        Register32::EBX => regs.ebx(),
                        Register32::ESP => regs.esp(),
                        Register32::EBP => regs.ebp(),
                        Register32::ESI => regs.esi(),
                        Register32::EDI => regs.edi(),
                        _ => 0,
                    },
                };
                let index_val = match index {
                    ScaledIndex::None => 0,
                    ScaledIndex::EaxScaled(scale) => regs.eax().wrapping_mul(scale.into()),
                    ScaledIndex::EcxScaled(scale) => regs.ecx().wrapping_mul(scale.into()),
                    ScaledIndex::EdxScaled(scale) => regs.edx().wrapping_mul(scale.into()),
                    ScaledIndex::EbxScaled(scale) => regs.ebx().wrapping_mul(scale.into()),
                    ScaledIndex::EbpScaled(scale) => regs.ebp().wrapping_mul(scale.into()),
                    ScaledIndex::EsiScaled(scale) => regs.esi().wrapping_mul(scale.into()),
                    ScaledIndex::EdiScaled(scale) => regs.edi().wrapping_mul(scale.into()),
                };
                base_val.wrapping_add(index_val).wrapping_add(*disp as u32)
            }
        };

        offset
    }

    pub fn calculate_effective_address(&self, base_segment: Register32, regs: &dyn Registers32) -> u32 {
        let offset = self.calculate(regs);

        let base_addr = match base_segment {
            Register32::ES => regs.es_base(),
            Register32::CS => regs.cs_base(),
            Register32::SS => regs.ss_base(),
            Register32::DS => regs.ds_base(),
            Register32::FS => regs.fs_base(),
            Register32::GS => regs.gs_base(),
            _ => 0,
        };

        base_addr.wrapping_add(offset)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum SegmentRegister {
    #[default]
    CS,
    SS,
    DS,
    ES,
    FS,
    GS,
}

impl From<SegmentRegister> for iced_x86::Register {
    fn from(value: SegmentRegister) -> Self {
        match value {
            SegmentRegister::ES => iced_x86::Register::ES,
            SegmentRegister::CS => iced_x86::Register::CS,
            SegmentRegister::SS => iced_x86::Register::SS,
            SegmentRegister::DS => iced_x86::Register::DS,
            SegmentRegister::FS => iced_x86::Register::FS,
            SegmentRegister::GS => iced_x86::Register::GS,
        }
    }
}

impl From<SegmentRegister> for moo::types::MooSegmentRegister {
    fn from(value: SegmentRegister) -> Self {
        match value {
            SegmentRegister::ES => moo::types::MooSegmentRegister::ES,
            SegmentRegister::CS => moo::types::MooSegmentRegister::CS,
            SegmentRegister::SS => moo::types::MooSegmentRegister::SS,
            SegmentRegister::DS => moo::types::MooSegmentRegister::DS,
            SegmentRegister::FS => moo::types::MooSegmentRegister::FS,
            SegmentRegister::GS => moo::types::MooSegmentRegister::GS,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Register16 {
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
    ES,
    CS,
    SS,
    DS,
    FS,
    GS,
    PC,
    #[default]
    InvalidRegister,
}

impl Display for Register16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Register16::AX => write!(f, "ax"),
            Register16::CX => write!(f, "cx"),
            Register16::DX => write!(f, "dx"),
            Register16::BX => write!(f, "bx"),
            Register16::SP => write!(f, "sp"),
            Register16::BP => write!(f, "bp"),
            Register16::SI => write!(f, "si"),
            Register16::DI => write!(f, "di"),
            Register16::ES => write!(f, "es"),
            Register16::CS => write!(f, "cs"),
            Register16::SS => write!(f, "ss"),
            Register16::DS => write!(f, "ds"),
            Register16::FS => write!(f, "fs"),
            Register16::GS => write!(f, "gs"),
            Register16::PC => write!(f, "ip"),
            Register16::InvalidRegister => write!(f, "invalid"),
        }
    }
}

impl From<iced_x86::Register> for Register16 {
    fn from(value: iced_x86::Register) -> Self {
        match value {
            iced_x86::Register::AX => Register16::AX,
            iced_x86::Register::CX => Register16::CX,
            iced_x86::Register::DX => Register16::DX,
            iced_x86::Register::BX => Register16::BX,
            iced_x86::Register::SP => Register16::SP,
            iced_x86::Register::BP => Register16::BP,
            iced_x86::Register::SI => Register16::SI,
            iced_x86::Register::DI => Register16::DI,
            iced_x86::Register::ES => Register16::ES,
            iced_x86::Register::CS => Register16::CS,
            iced_x86::Register::SS => Register16::SS,
            iced_x86::Register::DS => Register16::DS,
            iced_x86::Register::FS => Register16::FS,
            iced_x86::Register::GS => Register16::GS,
            _ => Register16::InvalidRegister,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Register32 {
    EAX,
    ECX,
    EDX,
    EBX,
    ESP,
    EBP,
    ESI,
    EDI,
    ES,
    CS,
    SS,
    DS,
    FS,
    GS,
    PC,
    InvalidRegister,
}

impl TryFrom<&Register32> for SegmentRegister {
    type Error = ();

    fn try_from(value: &Register32) -> Result<Self, Self::Error> {
        match value {
            Register32::ES => Ok(SegmentRegister::ES),
            Register32::CS => Ok(SegmentRegister::CS),
            Register32::SS => Ok(SegmentRegister::SS),
            Register32::DS => Ok(SegmentRegister::DS),
            Register32::FS => Ok(SegmentRegister::FS),
            Register32::GS => Ok(SegmentRegister::GS),
            _ => Err(()),
        }
    }
}

impl TryFrom<&Register16> for SegmentRegister {
    type Error = ();

    fn try_from(value: &Register16) -> Result<Self, Self::Error> {
        match value {
            Register16::ES => Ok(SegmentRegister::ES),
            Register16::CS => Ok(SegmentRegister::CS),
            Register16::SS => Ok(SegmentRegister::SS),
            Register16::DS => Ok(SegmentRegister::DS),
            Register16::FS => Ok(SegmentRegister::FS),
            Register16::GS => Ok(SegmentRegister::GS),
            _ => Err(()),
        }
    }
}

impl From<iced_x86::Register> for Register32 {
    fn from(value: iced_x86::Register) -> Self {
        match value {
            iced_x86::Register::EAX => Register32::EAX,
            iced_x86::Register::ECX => Register32::ECX,
            iced_x86::Register::EDX => Register32::EDX,
            iced_x86::Register::EBX => Register32::EBX,
            iced_x86::Register::ESP => Register32::ESP,
            iced_x86::Register::EBP => Register32::EBP,
            iced_x86::Register::ESI => Register32::ESI,
            iced_x86::Register::EDI => Register32::EDI,
            iced_x86::Register::ES => Register32::ES,
            iced_x86::Register::CS => Register32::CS,
            iced_x86::Register::SS => Register32::SS,
            iced_x86::Register::DS => Register32::DS,
            iced_x86::Register::FS => Register32::FS,
            iced_x86::Register::GS => Register32::GS,
            _ => Register32::InvalidRegister,
        }
    }
}

impl Display for Register32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Register32::EAX => write!(f, "eax"),
            Register32::ECX => write!(f, "ecx"),
            Register32::EDX => write!(f, "edx"),
            Register32::EBX => write!(f, "ebx"),
            Register32::ESP => write!(f, "esp"),
            Register32::EBP => write!(f, "ebp"),
            Register32::ESI => write!(f, "esi"),
            Register32::EDI => write!(f, "edi"),
            Register32::ES => write!(f, "es"),
            Register32::CS => write!(f, "cs"),
            Register32::SS => write!(f, "ss"),
            Register32::DS => write!(f, "ds"),
            Register32::FS => write!(f, "fs"),
            Register32::GS => write!(f, "gs"),
            Register32::PC => write!(f, "eip"),
            Register32::InvalidRegister => write!(f, "invalid"),
        }
    }
}

pub const REGISTER8_LUT: [Register8; 8] = [
    Register8::AL,
    Register8::CL,
    Register8::DL,
    Register8::BL,
    Register8::AH,
    Register8::CH,
    Register8::DH,
    Register8::BH,
];

pub const REGISTER16_LUT: [Register16; 8] = [
    Register16::AX,
    Register16::CX,
    Register16::DX,
    Register16::BX,
    Register16::SP,
    Register16::BP,
    Register16::SI,
    Register16::DI,
];

pub const SREGISTER16_LUT: [Register16; 8] = [
    Register16::ES,
    Register16::CS,
    Register16::SS,
    Register16::DS,
    Register16::ES,
    Register16::CS,
    Register16::SS,
    Register16::DS,
];

pub const SREGISTER32_LUT: [Register32; 8] = [
    Register32::ES,
    Register32::CS,
    Register32::SS,
    Register32::DS,
    Register32::FS,
    Register32::GS,
    Register32::SS,
    Register32::DS,
];

pub const REGISTER32_LUT: [Register32; 8] = [
    Register32::EAX,
    Register32::ECX,
    Register32::EDX,
    Register32::EBX,
    Register32::ESP,
    Register32::EBP,
    Register32::ESI,
    Register32::EDI,
];

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Register8 {
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,
}

#[derive(Debug)]
pub enum BusStatusByte {
    V1(u8),
    V2(u8),
    V3(u8),
}

impl TryFrom<BusStatusByte> for BusOpType {
    type Error = ();

    fn try_from(value: BusStatusByte) -> Result<Self, Self::Error> {
        match value {
            BusStatusByte::V1(v) => match v & 0x7 {
                0b00 => Ok(BusOpType::CodeRead),
                0b001 => Ok(BusOpType::IoRead),
                0b010 => Ok(BusOpType::IoWrite),
                0b101 => Ok(BusOpType::MemRead),
                0b110 => Ok(BusOpType::MemWrite),
                _ => Err(()),
            },
            BusStatusByte::V2(v) => match v & 0xF {
                0b0101 => Ok(BusOpType::MemRead),
                0b0110 => Ok(BusOpType::MemWrite),
                0b1001 => Ok(BusOpType::IoRead),
                0b1010 => Ok(BusOpType::IoWrite),
                0b1101 => Ok(BusOpType::CodeRead),
                _ => Err(()),
            },
            BusStatusByte::V3(v) => match v & 0x07 {
                0b010 => Ok(BusOpType::IoRead),
                0b011 => Ok(BusOpType::IoWrite),
                0b100 => Ok(BusOpType::CodeRead),
                0b110 => Ok(BusOpType::MemRead),
                0b111 => Ok(BusOpType::MemWrite),
                _ => Err(()),
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BusOpType {
    CodeRead,
    MemRead,
    MemWrite,
    IoRead,
    IoWrite,
}

#[derive(Copy, Clone, Debug)]
pub struct BusOp {
    pub idx: usize,
    pub op_type: BusOpType,
    pub addr: u32,
    pub bhe: bool,
    pub data: u16,
    pub flags: u8,
}

impl BusOp {
    pub const FLAG_PAIRED: u8 = 0x01;
    pub const FLAG_STACK: u8 = 0x02;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct EffectiveAddress {
    pub(crate) base_segment: SegmentRegister,
    pub(crate) base_segment_selector: u16,
    pub(crate) base_segment_address: u32,
    pub(crate) base_segment_limit: u32,
    pub(crate) offset: u32,
    pub(crate) linear_address: u32,
    pub(crate) physical_address: u32,
}

impl EffectiveAddress {
    pub fn trace_log(&self, context: &mut TestContext) {
        trace_banner!(context);
        trace_log!(context, ">>> Effective Address Calculation:");
        trace_log!(
            context,
            ">>>  Base Segment:       {:?} [selector={:04X} base={:08X} limit={:08X}]",
            self.base_segment,
            self.base_segment_selector,
            self.base_segment_address,
            self.base_segment_limit
        );
        trace_log!(context, ">>>  Offset:             {:08X}", self.offset);
        trace_log!(context, ">>>  Linear Address:     {:08X}", self.linear_address);
        trace_log!(context, ">>>  Physical Address:   {:08X}", self.physical_address);
        trace_banner!(context);
        // No paging
    }

    pub fn into_moo(self) -> MooEffectiveAddress {
        MooEffectiveAddress {
            base_segment: self.base_segment.into(),
            base_selector: self.base_segment_selector,
            base_address: self.base_segment_address,
            base_limit: self.base_segment_limit,
            offset: self.offset,
            linear_address: self.linear_address,
            physical_address: self.physical_address,
        }
    }
}
