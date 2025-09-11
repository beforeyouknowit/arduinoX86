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
//! Module for handling ModRM bytes in x86 instructions.
//! [ModRmByte] is designed to be `'static`, so that reading a ModRM byte resolves to
//! a static reference in a const table of [ModRmByte], plus a displacement which can
//! later be passed to `ModRmByte::addressing_mode()` for resolution.

use crate::cpu_common::{
    AddressOffset16,
    AddressOffset32,
    BaseRegister,
    Displacement,
    Register16,
    Register32,
    Register8,
    ScaledIndex,
    SibScale,
    REGISTER16_LUT,
    REGISTER32_LUT,
    REGISTER8_LUT,
    SREGISTER16_LUT,
    SREGISTER32_LUT,
};

pub const MODRM_REG_MASK: u8 = 0b00_111_000;
pub const MODRM_ADDR_MASK: u8 = 0b11_000_111;

pub const SIB_INDEX_MASK: u8 = 0b11_111_000;
pub const SIB_BASE_MASK: u8 = 0b00_000_111;
pub const SIB_DISPLACEMENT: u8 = 0b00_000_101;

//pub const MODRM_MOD_MASK:          u8 = 0b11_000_000;

// 16-bit modrm bitmasks
const MODRM_ADDR_BX_SI: u8 = 0b00_000_000;
const MODRM_ADDR_BX_DI: u8 = 0b00_000_001;
const MODRM_ADDR_BP_SI: u8 = 0b00_000_010;
const MODRM_ADDR_BP_DI: u8 = 0b00_000_011;
const MODRM_ADDR_SI: u8 = 0b00_000_100;
const MODRM_ADDR_DI: u8 = 0b00_000_101;
const MODRM_ADDR_DISP16: u8 = 0b00_000_110;
const MODRM_ADDR_BX: u8 = 0b00_000_111;

const MODRM_ADDR_BX_SI_DISP8: u8 = 0b01_000_000;
const MODRM_ADDR_BX_DI_DISP8: u8 = 0b01_000_001;
const MODRM_ADDR_BP_SI_DISP8: u8 = 0b01_000_010;
const MODRM_ADDR_BP_DI_DISP8: u8 = 0b01_000_011;
const MODRM_ADDR_SI_DISP8: u8 = 0b01_000_100;
const MODRM_ADDR_DI_DISP8: u8 = 0b01_000_101;
const MODRM_ADDR_BP_DISP8: u8 = 0b01_000_110;
const MODRM_ADDR_BX_DISP8: u8 = 0b01_000_111;

const MODRM_ADDR_BX_SI_DISP16: u8 = 0b10_000_000;
const MODRM_ADDR_BX_DI_DISP16: u8 = 0b10_000_001;
const MODRM_ADDR_BP_SI_DISP16: u8 = 0b10_000_010;
const MODRM_ADDR_BP_DI_DISP16: u8 = 0b10_000_011;
const MODRM_ADDR_SI_DISP16: u8 = 0b10_000_100;
const MODRM_ADDR_DI_DISP16: u8 = 0b10_000_101;
const MODRM_ADDR_BP_DISP16: u8 = 0b10_000_110;
const MODRM_ADDR_BX_DISP16: u8 = 0b10_000_111;

// 32-bit modrm bitmasks
const MODRM_ADDR_EAX: u8 = 0b00_000_000;
const MODRM_ADDR_ECX: u8 = 0b00_000_001;
const MODRM_ADDR_EDX: u8 = 0b00_000_010;
const MODRM_ADDR_EBX: u8 = 0b00_000_011;
const MODRM_ADDR_SIB0: u8 = 0b00_000_100;
const MODRM_ADDR_DISP32: u8 = 0b00_000_101;
const MODRM_ADDR_ESI: u8 = 0b00_000_110;
const MODRM_ADDR_EDI: u8 = 0b00_000_111;

const MODRM_ADDR_EAX_DISP8: u8 = 0b01_000_000;
const MODRM_ADDR_ECX_DISP8: u8 = 0b01_000_001;
const MODRM_ADDR_EDX_DISP8: u8 = 0b01_000_010;
const MODRM_ADDR_EBX_DISP8: u8 = 0b01_000_011;
const MODRM_ADDR_SIB1: u8 = 0b01_000_100;
const MODRM_ADDR_EBP_DISP8: u8 = 0b01_000_101;
const MODRM_ADDR_ESI_DISP8: u8 = 0b01_000_110;
const MODRM_ADDR_EDI_DISP8: u8 = 0b01_000_111;

const MODRM_ADDR_EAX_DISP32: u8 = 0b10_000_000;
const MODRM_ADDR_ECX_DISP32: u8 = 0b10_000_001;
const MODRM_ADDR_EDX_DISP32: u8 = 0b10_000_010;
const MODRM_ADDR_EBX_DISP32: u8 = 0b10_000_011;
const MODRM_ADDR_SIB2: u8 = 0b10_000_100;
const MODRM_ADDR_EBP_DISP32: u8 = 0b10_000_101;
const MODRM_ADDR_ESI_DISP32: u8 = 0b10_000_110;
const MODRM_ADDR_EDI_DISP32: u8 = 0b10_000_111;

// 32-bit SIB bitmasks
const SIB_EAX: u8 = 0b00_000_000;
const SIB_ECX: u8 = 0b00_001_000;
const SIB_EDX: u8 = 0b00_010_000;
const SIB_EBX: u8 = 0b00_011_000;
const SIB_NONE0: u8 = 0b00_100_000;
const SIB_EBP: u8 = 0b00_101_000;
const SIB_ESI: u8 = 0b00_110_000;
const SIB_EDI: u8 = 0b00_111_000;

const SIB_EAX_S2: u8 = 0b01_000_000;
const SIB_ECX_S2: u8 = 0b01_001_000;
const SIB_EDX_S2: u8 = 0b01_010_000;
const SIB_EBX_S2: u8 = 0b01_011_000;
const SIB_NONE1: u8 = 0b01_100_000;
const SIB_EBP_S2: u8 = 0b01_101_000;
const SIB_ESI_S2: u8 = 0b01_110_000;
const SIB_EDI_S2: u8 = 0b01_111_000;

const SIB_EAX_S4: u8 = 0b10_000_000;
const SIB_ECX_S4: u8 = 0b10_001_000;
const SIB_EDX_S4: u8 = 0b10_010_000;
const SIB_EBX_S4: u8 = 0b10_011_000;
const SIB_NONE2: u8 = 0b10_100_000;
const SIB_EBP_S4: u8 = 0b10_101_000;
const SIB_ESI_S4: u8 = 0b10_110_000;
const SIB_EDI_S4: u8 = 0b10_111_000;

const SIB_EAX_S8: u8 = 0b11_000_000;
const SIB_ECX_S8: u8 = 0b11_001_000;
const SIB_EDX_S8: u8 = 0b11_010_000;
const SIB_EBX_S8: u8 = 0b11_011_000;
const SIB_NONE3: u8 = 0b11_100_000;
const SIB_EBP_S8: u8 = 0b11_101_000;
const SIB_ESI_S8: u8 = 0b11_110_000;
const SIB_EDI_S8: u8 = 0b11_111_000;

#[derive(Copy, Clone)]
pub struct ModRmByte16 {
    _byte: u8,
    b_mod: u8,
    b_reg: u8,
    b_rm: u8,
    disp: Displacement,
    addressing_mode: AddressOffset16,
}

const MODRM16_TABLE: [ModRmByte16; 256] = {
    let mut table: [ModRmByte16; 256] = [ModRmByte16 {
        _byte: 0,
        b_mod: 0,
        b_reg: 0,
        b_rm: 0,
        disp: Displacement::NoDisp,
        addressing_mode: AddressOffset16::BxSi,
    }; 256];
    let mut byte = 0;

    loop {
        let mut displacement = Displacement::NoDisp;

        let b_mod = (byte >> 6) & 0x03;

        match b_mod {
            0b00 => {
                // Addressing mode [disp16] is a single mode of 0b00
                if byte & MODRM_ADDR_MASK == MODRM_ADDR_DISP16 {
                    displacement = Displacement::Pending16;
                }
            }
            0b01 => {
                // 0b01 signifies an 8 bit displacement (sign-extended to 16)
                displacement = Displacement::Pending8;
            }
            0b10 => {
                // 0b10 signifies a 16 bit displacement
                displacement = Displacement::Pending16;
            }
            _ => displacement = Displacement::NoDisp,
        }

        // Set the addressing mode based on the combination of Mod and R/M bitfields + Displacement.
        let addressing_mode = match byte & MODRM_ADDR_MASK {
            MODRM_ADDR_BX_SI => AddressOffset16::BxSi,
            MODRM_ADDR_BX_DI => AddressOffset16::BxDi,
            MODRM_ADDR_BP_SI => AddressOffset16::BpSi,
            MODRM_ADDR_BP_DI => AddressOffset16::BpDi,
            MODRM_ADDR_SI => AddressOffset16::Si,
            MODRM_ADDR_DI => AddressOffset16::Di,
            MODRM_ADDR_DISP16 => AddressOffset16::Disp16(0),
            MODRM_ADDR_BX => AddressOffset16::Bx,
            MODRM_ADDR_BX_SI_DISP8 => AddressOffset16::BxSiDisp8(0),
            MODRM_ADDR_BX_DI_DISP8 => AddressOffset16::BxDiDisp8(0),
            MODRM_ADDR_BP_SI_DISP8 => AddressOffset16::BpSiDisp8(0),
            MODRM_ADDR_BP_DI_DISP8 => AddressOffset16::BpDiDisp8(0),
            MODRM_ADDR_SI_DISP8 => AddressOffset16::SiDisp8(0),
            MODRM_ADDR_DI_DISP8 => AddressOffset16::DiDisp8(0),
            MODRM_ADDR_BP_DISP8 => AddressOffset16::BpDisp8(0),
            MODRM_ADDR_BX_DISP8 => AddressOffset16::BxDisp8(0),
            MODRM_ADDR_BX_SI_DISP16 => AddressOffset16::BxSiDisp16(0),
            MODRM_ADDR_BX_DI_DISP16 => AddressOffset16::BxDiDisp16(0),
            MODRM_ADDR_BP_SI_DISP16 => AddressOffset16::BpSiDisp16(0),
            MODRM_ADDR_BP_DI_DISP16 => AddressOffset16::BpDiDisp16(0),
            MODRM_ADDR_SI_DISP16 => AddressOffset16::SiDisp16(0),
            MODRM_ADDR_DI_DISP16 => AddressOffset16::DiDisp16(0),
            MODRM_ADDR_BP_DISP16 => AddressOffset16::BpDisp16(0),
            MODRM_ADDR_BX_DISP16 => AddressOffset16::BxDisp16(0),
            _ => AddressOffset16::None,
        };

        // 'REG' field specifies either register operand or opcode extension. There's no way
        // to know without knowing the opcode, which we don't
        let b_reg: u8 = (byte >> 3) & 0x07;

        // 'R/M' field is last three bits
        let b_rm: u8 = byte & 0x07;

        table[byte as usize] = ModRmByte16 {
            _byte: byte,
            b_mod,
            b_reg,
            b_rm,
            disp: displacement,
            addressing_mode,
        };

        if byte < 255 {
            byte += 1;
        }
        else {
            break;
        }
    }

    table
};

#[derive(Copy, Clone)]
pub struct ModRmByte32 {
    _byte: u8,
    b_mod: u8,
    b_reg: u8,
    b_rm: u8,
    addressing_mode: AddressOffset32,
}

const MODRM32_TABLE: [ModRmByte32; 256] = {
    let mut table: [ModRmByte32; 256] = [ModRmByte32 {
        _byte: 0,
        b_mod: 0,
        b_reg: 0,
        b_rm: 0,
        addressing_mode: AddressOffset32::Eax,
    }; 256];
    let mut byte = 0;

    loop {
        let mut displacement = Displacement::NoDisp;

        let b_mod = (byte >> 6) & 0x03;

        match b_mod {
            0b00 => {
                // Addressing mode [disp32] is a single mode of 0b00
                if byte & MODRM_ADDR_MASK == MODRM_ADDR_DISP32 {
                    displacement = Displacement::Pending32;
                }
            }
            0b01 => {
                // 0b01 signifies an 8 bit displacement (sign-extended to 16)
                displacement = Displacement::Pending8;
            }
            0b10 => {
                // 0b10 signifies a 32 bit displacement
                displacement = Displacement::Pending32;
            }
            _ => displacement = Displacement::NoDisp,
        }

        let addressing_mode = match byte & MODRM_ADDR_MASK {
            MODRM_ADDR_EAX => AddressOffset32::Eax,
            MODRM_ADDR_ECX => AddressOffset32::Ecx,
            MODRM_ADDR_EDX => AddressOffset32::Edx,
            MODRM_ADDR_EBX => AddressOffset32::Ebx,
            MODRM_ADDR_SIB0 => AddressOffset32::SibPending,
            MODRM_ADDR_DISP32 => AddressOffset32::Disp32(0),
            MODRM_ADDR_ESI => AddressOffset32::Esi,
            MODRM_ADDR_EDI => AddressOffset32::Edi,

            MODRM_ADDR_EAX_DISP8 => AddressOffset32::EaxDisp8(0),
            MODRM_ADDR_ECX_DISP8 => AddressOffset32::EcxDisp8(0),
            MODRM_ADDR_EDX_DISP8 => AddressOffset32::EdxDisp8(0),
            MODRM_ADDR_EBX_DISP8 => AddressOffset32::EbxDisp8(0),
            MODRM_ADDR_SIB1 => AddressOffset32::SibPending,
            MODRM_ADDR_EBP_DISP8 => AddressOffset32::EbpDisp8(0),
            MODRM_ADDR_ESI_DISP8 => AddressOffset32::EsiDisp8(0),
            MODRM_ADDR_EDI_DISP8 => AddressOffset32::EdiDisp8(0),

            MODRM_ADDR_EAX_DISP32 => AddressOffset32::EaxDisp32(0),
            MODRM_ADDR_ECX_DISP32 => AddressOffset32::EcxDisp32(0),
            MODRM_ADDR_EDX_DISP32 => AddressOffset32::EdxDisp32(0),
            MODRM_ADDR_EBX_DISP32 => AddressOffset32::EbxDisp32(0),
            MODRM_ADDR_SIB2 => AddressOffset32::SibPending,
            MODRM_ADDR_EBP_DISP32 => AddressOffset32::EbpDisp32(0),
            MODRM_ADDR_ESI_DISP32 => AddressOffset32::EsiDisp32(0),
            MODRM_ADDR_EDI_DISP32 => AddressOffset32::EdiDisp32(0),
            _ => AddressOffset32::None,
        };

        // 'REG' field specifies either register operand or opcode extension. There's no way
        // to know without knowing the opcode, which we don't
        let b_reg: u8 = (byte >> 3) & 0x07;

        // 'R/M' field is last three bits
        let b_rm: u8 = byte & 0x07;

        table[byte as usize] = ModRmByte32 {
            _byte: byte,
            b_mod,
            b_reg,
            b_rm,
            addressing_mode,
        };

        if byte < 255 {
            byte += 1;
        }
        else {
            break;
        }
    }

    table
};

#[derive(Copy, Clone)]
pub struct SibByte {
    _byte: u8,
    modrm_mod: u8,
    b_ss: u8,
    b_base: u8,
    b_idx: u8,
    addressing_mode: AddressOffset32,
}

const SIB_TABLE: [SibByte; 768] = {
    let mut table: [SibByte; 768] = [SibByte {
        _byte: 0,
        modrm_mod: 0,
        b_ss: 0,
        b_base: 0,
        b_idx: 0,
        addressing_mode: AddressOffset32::Eax,
    }; 768];
    let mut byte = 0;

    let mut modrm_mod = 0;

    loop {
        // Match the SIB scale factor
        let b_ss = (byte >> 6) & 0x03;

        let scaled_index = match byte & SIB_INDEX_MASK {
            SIB_EAX => ScaledIndex::EaxScaled(SibScale::One),
            SIB_ECX => ScaledIndex::EcxScaled(SibScale::One),
            SIB_EDX => ScaledIndex::EdxScaled(SibScale::One),
            SIB_EBX => ScaledIndex::EbxScaled(SibScale::One),
            SIB_NONE0 => ScaledIndex::None,
            SIB_EBP => ScaledIndex::EbpScaled(SibScale::One),
            SIB_ESI => ScaledIndex::EsiScaled(SibScale::One),
            SIB_EDI => ScaledIndex::EdiScaled(SibScale::One),

            SIB_EAX_S2 => ScaledIndex::EaxScaled(SibScale::Two),
            SIB_ECX_S2 => ScaledIndex::EcxScaled(SibScale::Two),
            SIB_EDX_S2 => ScaledIndex::EdxScaled(SibScale::Two),
            SIB_EBX_S2 => ScaledIndex::EbxScaled(SibScale::Two),
            SIB_NONE1 => ScaledIndex::None,
            SIB_EBP_S2 => ScaledIndex::EbpScaled(SibScale::Two),
            SIB_ESI_S2 => ScaledIndex::EsiScaled(SibScale::Two),
            SIB_EDI_S2 => ScaledIndex::EdiScaled(SibScale::Two),

            SIB_EAX_S4 => ScaledIndex::EaxScaled(SibScale::Four),
            SIB_ECX_S4 => ScaledIndex::EcxScaled(SibScale::Four),
            SIB_EDX_S4 => ScaledIndex::EdxScaled(SibScale::Four),
            SIB_EBX_S4 => ScaledIndex::EbxScaled(SibScale::Four),
            SIB_NONE2 => ScaledIndex::None,
            SIB_EBP_S4 => ScaledIndex::EbpScaled(SibScale::Four),
            SIB_ESI_S4 => ScaledIndex::EsiScaled(SibScale::Four),
            SIB_EDI_S4 => ScaledIndex::EdiScaled(SibScale::Four),

            SIB_EAX_S8 => ScaledIndex::EaxScaled(SibScale::Eight),
            SIB_ECX_S8 => ScaledIndex::EcxScaled(SibScale::Eight),
            SIB_EDX_S8 => ScaledIndex::EdxScaled(SibScale::Eight),
            SIB_EBX_S8 => ScaledIndex::EbxScaled(SibScale::Eight),
            SIB_NONE3 => ScaledIndex::None,
            SIB_EBP_S8 => ScaledIndex::EbpScaled(SibScale::Eight),
            SIB_ESI_S8 => ScaledIndex::EsiScaled(SibScale::Eight),
            SIB_EDI_S8 => ScaledIndex::EdiScaled(SibScale::Eight),
            _ => unreachable!(),
        };

        let b_base: u8 = byte & 0x07;
        let base_reg = REGISTER32_LUT[b_base as usize];
        let b_idx: u8 = (byte >> 3) & 0x07;

        let addressing_mode = if byte & SIB_BASE_MASK == SIB_DISPLACEMENT {
            match modrm_mod {
                0b00 => AddressOffset32::SibDisp32(BaseRegister::None, scaled_index, 0),
                0b01 => AddressOffset32::SibDisp8Ebp(BaseRegister::Some(Register32::EBP), scaled_index, 0),
                0b10 => AddressOffset32::SibDisp32Ebp(BaseRegister::Some(Register32::EBP), scaled_index, 0),
                _ => unreachable!(),
            }
        }
        else {
            AddressOffset32::Sib(BaseRegister::Some(base_reg), scaled_index)
        };

        table[((modrm_mod as usize) * 256) + byte as usize] = SibByte {
            _byte: byte,
            modrm_mod,
            b_ss,
            b_base,
            b_idx,
            addressing_mode,
        };

        if byte < 255 {
            byte += 1;
        }
        else {
            byte = 0;
            modrm_mod += 1;
            if modrm_mod > 2 {
                break;
            }
        }
    }

    table
};

impl ModRmByte16 {
    pub fn default_ref() -> &'static ModRmByte16 {
        &MODRM16_TABLE[0]
    }

    /// Read the modrm byte and look up the appropriate value from the modrm table.
    /// Load any displacement, then return modrm struct and size of modrm + displacement.
    pub fn read(byte: u8) -> &'static ModRmByte16 {
        log::debug!("ModRmByte16::read({:#02x})", byte);
        &MODRM16_TABLE[byte as usize]
    }

    /// Return the 'mod' field (top two bits) of the modrm byte.
    pub fn mod_value(&self) -> u8 {
        self.b_mod
    }

    // Interpret the 'R/M' field as an 8 bit register selector
    #[inline(always)]
    pub fn op1_reg8(&self) -> Register8 {
        REGISTER8_LUT[self.b_rm as usize]
    }
    // Interpret the 'R/M' field as a 16 bit register selector
    #[inline(always)]
    pub fn op1_reg16(&self) -> Register16 {
        REGISTER16_LUT[self.b_rm as usize]
    }
    // Interpret the 'REG' field as an 8 bit register selector
    #[inline(always)]
    pub fn op2_reg8(&self) -> Register8 {
        REGISTER8_LUT[self.b_reg as usize]
    }
    // Interpret the 'REG' field as a 16 bit register selector
    #[inline(always)]
    pub fn op2_reg16(&self) -> Register16 {
        REGISTER16_LUT[self.b_reg as usize]
    }
    // Interpret the 'REG' field as a 16 bit segment register selector
    #[inline(always)]
    pub fn op2_segmentreg16(&self) -> Register16 {
        SREGISTER16_LUT[self.b_reg as usize]
    }
    // Interpret the 'REG' field as a 3 bit opcode extension
    #[inline(always)]
    pub fn op_extension(&self) -> u8 {
        self.b_reg
    }
    // Return whether the modrm byte specifies a memory addressing mode
    #[inline(always)]
    pub fn is_addressing_mode(&self) -> bool {
        self.b_mod != 0b11
    }
    /// Produce an [AddressOffset16] enum with the provided [Displacement] inserted.
    #[inline(always)]
    pub fn address_offset(&self, displacement: Displacement) -> AddressOffset16 {
        match self.addressing_mode {
            AddressOffset16::Disp16(_) => AddressOffset16::Disp16(displacement.into()),
            AddressOffset16::BxSiDisp8(_) => AddressOffset16::BxSiDisp8(displacement.into()),
            AddressOffset16::BxDiDisp8(_) => AddressOffset16::BxDiDisp8(displacement.into()),
            AddressOffset16::BpSiDisp8(_) => AddressOffset16::BpSiDisp8(displacement.into()),
            AddressOffset16::BpDiDisp8(_) => AddressOffset16::BpDiDisp8(displacement.into()),
            AddressOffset16::SiDisp8(_) => AddressOffset16::SiDisp8(displacement.into()),
            AddressOffset16::DiDisp8(_) => AddressOffset16::DiDisp8(displacement.into()),
            AddressOffset16::BpDisp8(_) => AddressOffset16::BpDisp8(displacement.into()),
            AddressOffset16::BxDisp8(_) => AddressOffset16::BxDisp8(displacement.into()),
            AddressOffset16::BxSiDisp16(_) => AddressOffset16::BxSiDisp16(displacement.into()),
            AddressOffset16::BxDiDisp16(_) => AddressOffset16::BxDiDisp16(displacement.into()),
            AddressOffset16::BpSiDisp16(_) => AddressOffset16::BpSiDisp16(displacement.into()),
            AddressOffset16::BpDiDisp16(_) => AddressOffset16::BpDiDisp16(displacement.into()),
            AddressOffset16::SiDisp16(_) => AddressOffset16::SiDisp16(displacement.into()),
            AddressOffset16::DiDisp16(_) => AddressOffset16::DiDisp16(displacement.into()),
            AddressOffset16::BpDisp16(_) => AddressOffset16::BpDisp16(displacement.into()),
            AddressOffset16::BxDisp16(_) => AddressOffset16::BxDisp16(displacement.into()),
            _ => self.addressing_mode,
        }
    }
}

impl ModRmByte32 {
    pub fn default_ref() -> &'static ModRmByte32 {
        &MODRM32_TABLE[0]
    }

    /// Read the modrm byte and look up the appropriate value from the modrm table.
    /// Load any displacement, then return modrm struct and size of modrm + displacement.
    pub fn read(byte: u8) -> &'static ModRmByte32 {
        &MODRM32_TABLE[byte as usize]
    }

    /// Return the 'mod' field (top two bits) of the modrm byte.
    pub fn mod_value(&self) -> u8 {
        self.b_mod
    }

    // Interpret the 'R/M' field as an 8 bit register selector
    #[inline(always)]
    pub fn op1_reg8(&self) -> Register8 {
        REGISTER8_LUT[self.b_rm as usize]
    }
    // Interpret the 'R/M' field as a 16 bit register selector
    #[inline(always)]
    pub fn op1_reg32(&self) -> Register32 {
        REGISTER32_LUT[self.b_rm as usize]
    }
    // Interpret the 'REG' field as an 8 bit register selector
    #[inline(always)]
    pub fn op2_reg8(&self) -> Register8 {
        REGISTER8_LUT[self.b_reg as usize]
    }
    // Interpret the 'REG' field as a 16 bit register selector
    #[inline(always)]
    pub fn op2_reg32(&self) -> Register32 {
        REGISTER32_LUT[self.b_reg as usize]
    }
    // Interpret the 'REG' field as a 32 bit segment register selector
    #[inline(always)]
    pub fn op2_segmentreg32(&self) -> Register32 {
        SREGISTER32_LUT[self.b_reg as usize]
    }
    // Interpret the 'REG' field as a 3 bit opcode extension
    #[inline(always)]
    pub fn op_extension(&self) -> u8 {
        self.b_reg
    }
    // Return whether the modrm byte specifies a memory addressing mode
    #[inline(always)]
    pub fn is_addressing_mode(&self) -> bool {
        self.b_mod != 0b11
    }
    /// Produce an [AddressOffset32] enum with the provided [Displacement] inserted.
    #[inline(always)]
    pub fn address_offset(&self, displacement: Displacement) -> AddressOffset32 {
        match self.addressing_mode {
            AddressOffset32::Disp32(_) => AddressOffset32::Disp32(displacement.into()),
            AddressOffset32::EaxDisp8(_) => AddressOffset32::EaxDisp8(displacement.into()),
            AddressOffset32::EcxDisp8(_) => AddressOffset32::EcxDisp8(displacement.into()),
            AddressOffset32::EdxDisp8(_) => AddressOffset32::EdxDisp8(displacement.into()),
            AddressOffset32::EbxDisp8(_) => AddressOffset32::EbxDisp8(displacement.into()),
            AddressOffset32::EbpDisp8(_) => AddressOffset32::EbpDisp8(displacement.into()),
            AddressOffset32::EsiDisp8(_) => AddressOffset32::EsiDisp8(displacement.into()),
            AddressOffset32::EdiDisp8(_) => AddressOffset32::EdiDisp8(displacement.into()),
            AddressOffset32::EaxDisp32(_) => AddressOffset32::EaxDisp32(displacement.into()),
            AddressOffset32::EcxDisp32(_) => AddressOffset32::EcxDisp32(displacement.into()),
            AddressOffset32::EdxDisp32(_) => AddressOffset32::EdxDisp32(displacement.into()),
            AddressOffset32::EbxDisp32(_) => AddressOffset32::EbxDisp32(displacement.into()),
            AddressOffset32::EbpDisp32(_) => AddressOffset32::EbpDisp32(displacement.into()),
            AddressOffset32::EsiDisp32(_) => AddressOffset32::EsiDisp32(displacement.into()),
            AddressOffset32::EdiDisp32(_) => AddressOffset32::EdiDisp32(displacement.into()),
            _ => self.addressing_mode,
        }
    }

    pub fn has_sib(&self) -> bool {
        matches!(self.addressing_mode, AddressOffset32::SibPending)
    }
}

impl SibByte {
    #[inline(always)]
    pub fn read(byte: u8, modrm_mod: u8) -> &'static SibByte {
        &SIB_TABLE[((modrm_mod & 0x03) as usize * 256) + byte as usize]
    }

    pub fn dump(&self) {
        for i in 0..768 {
            println!("{:02X}: {}", SIB_TABLE[i]._byte, SIB_TABLE[i].addressing_mode);
        }
    }

    pub fn byte(&self) -> u8 {
        self._byte
    }

    #[inline(always)]
    pub fn is_addressing_mode(&self) -> bool {
        true
    }

    /// Produce an [AddressOffset32] enum with the provided [Displacement] inserted.
    #[inline(always)]
    pub fn address_offset(&self, displacement: Displacement) -> AddressOffset32 {
        match self.addressing_mode {
            AddressOffset32::Sib(base, idx) => match displacement {
                Displacement::Disp8(d) => AddressOffset32::SibDisp8(base, idx, d),
                Displacement::Disp32(d) => AddressOffset32::SibDisp32(base, idx, d),
                _ => self.addressing_mode,
            },
            AddressOffset32::SibDisp8Ebp(base, idx, _) => AddressOffset32::SibDisp8Ebp(base, idx, displacement.into()),
            AddressOffset32::SibDisp32Ebp(base, idx, _) => {
                AddressOffset32::SibDisp32Ebp(base, idx, displacement.into())
            }
            AddressOffset32::SibDisp32(base, idx, _) => AddressOffset32::SibDisp32(base, idx, displacement.into()),
            _ => self.addressing_mode,
        }
    }
}
