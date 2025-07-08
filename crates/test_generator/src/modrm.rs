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

//! Module for handling ModRM bytes in x86 instructions.
//! [ModRmByte] is designed to be `'static`, so that reading a ModRM byte resolves to
//! a static reference in a const table of [ModRmByte], plus a displacement which can
//! later be passed to `ModRmByte::addressing_mode()` for resolution.

use crate::cpu_common::{
    AddressingMode, Displacement, Register16, Register8, REGISTER16_LUT, REGISTER8_LUT,
    SREGISTER_LUT,
};

pub const MODRM_REG_MASK: u8 = 0b00_111_000;
pub const MODRM_ADDR_MASK: u8 = 0b11_000_111;
//pub const MODRM_MOD_MASK:          u8 = 0b11_000_000;

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

#[derive(Copy, Clone)]
pub struct ModRmByte {
    _byte: u8,
    b_mod: u8,
    b_reg: u8,
    b_rm: u8,
    disp: Displacement,
    addressing_mode: AddressingMode,
}

const MODRM_TABLE: [ModRmByte; 256] = {
    let mut table: [ModRmByte; 256] = [ModRmByte {
        _byte: 0,
        b_mod: 0,
        b_reg: 0,
        b_rm: 0,
        disp: Displacement::NoDisp,
        addressing_mode: AddressingMode::BxSi,
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
        let (addressing_mode, displacement) = match byte & MODRM_ADDR_MASK {
            MODRM_ADDR_BX_SI => (AddressingMode::BxSi, Displacement::NoDisp),
            MODRM_ADDR_BX_DI => (AddressingMode::BxDi, Displacement::NoDisp),
            MODRM_ADDR_BP_SI => (AddressingMode::BpSi, Displacement::NoDisp),
            MODRM_ADDR_BP_DI => (AddressingMode::BpDi, Displacement::NoDisp),
            MODRM_ADDR_SI => (AddressingMode::Si, Displacement::NoDisp),
            MODRM_ADDR_DI => (AddressingMode::Di, Displacement::NoDisp),
            MODRM_ADDR_DISP16 => (AddressingMode::Disp16(displacement), displacement),
            MODRM_ADDR_BX => (AddressingMode::Bx, Displacement::NoDisp),
            MODRM_ADDR_BX_SI_DISP8 => (AddressingMode::BxSiDisp8(displacement), displacement),
            MODRM_ADDR_BX_DI_DISP8 => (AddressingMode::BxDiDisp8(displacement), displacement),
            MODRM_ADDR_BP_SI_DISP8 => (AddressingMode::BpSiDisp8(displacement), displacement),
            MODRM_ADDR_BP_DI_DISP8 => (AddressingMode::BpDiDisp8(displacement), displacement),
            MODRM_ADDR_SI_DISP8 => (AddressingMode::SiDisp8(displacement), displacement),
            MODRM_ADDR_DI_DISP8 => (AddressingMode::DiDisp8(displacement), displacement),
            MODRM_ADDR_BP_DISP8 => (AddressingMode::BpDisp8(displacement), displacement),
            MODRM_ADDR_BX_DISP8 => (AddressingMode::BxDisp8(displacement), displacement),
            MODRM_ADDR_BX_SI_DISP16 => (AddressingMode::BxSiDisp16(displacement), displacement),
            MODRM_ADDR_BX_DI_DISP16 => (AddressingMode::BxDiDisp16(displacement), displacement),
            MODRM_ADDR_BP_SI_DISP16 => (AddressingMode::BpSiDisp16(displacement), displacement),
            MODRM_ADDR_BP_DI_DISP16 => (AddressingMode::BpDiDisp16(displacement), displacement),
            MODRM_ADDR_SI_DISP16 => (AddressingMode::SiDisp16(displacement), displacement),
            MODRM_ADDR_DI_DISP16 => (AddressingMode::DiDisp16(displacement), displacement),
            MODRM_ADDR_BP_DISP16 => (AddressingMode::BpDisp16(displacement), displacement),
            MODRM_ADDR_BX_DISP16 => (AddressingMode::BxDisp16(displacement), displacement),
            _ => (AddressingMode::RegisterMode, Displacement::NoDisp),
        };

        // 'REG' field specifies either register operand or opcode extension. There's no way
        // to know without knowing the opcode, which we don't
        let b_reg: u8 = (byte >> 3) & 0x07;

        // 'R/M' field is last three bits
        let b_rm: u8 = byte & 0x07;

        table[byte as usize] = ModRmByte {
            _byte: byte,
            b_mod,
            b_reg,
            b_rm,
            disp: displacement,
            addressing_mode,
        };

        if byte < 255 {
            byte += 1;
        } else {
            break;
        }
    }

    table
};

impl ModRmByte {
    pub fn default_ref() -> &'static ModRmByte {
        &MODRM_TABLE[0]
    }

    /// Read the modrm byte and look up the appropriate value from the modrm table.
    /// Load any displacement, then return modrm struct and size of modrm + displacement.
    pub fn read(byte: u8) -> &'static ModRmByte {
        &MODRM_TABLE[byte as usize]
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
        SREGISTER_LUT[self.b_reg as usize]
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
    /// Produce an [AddressingMode] enum with the provided [Displacement] inserted.
    #[inline(always)]
    pub fn addressing_mode(&self, displacement: Displacement) -> AddressingMode {
        match self.addressing_mode {
            AddressingMode::Disp16(_) => AddressingMode::Disp16(displacement),
            AddressingMode::BxSiDisp8(_) => AddressingMode::BxSiDisp8(displacement),
            AddressingMode::BxDiDisp8(_) => AddressingMode::BxDiDisp8(displacement),
            AddressingMode::BpSiDisp8(_) => AddressingMode::BpSiDisp8(displacement),
            AddressingMode::BpDiDisp8(_) => AddressingMode::BpDiDisp8(displacement),
            AddressingMode::SiDisp8(_) => AddressingMode::SiDisp8(displacement),
            AddressingMode::DiDisp8(_) => AddressingMode::DiDisp8(displacement),
            AddressingMode::BpDisp8(_) => AddressingMode::BpDisp8(displacement),
            AddressingMode::BxDisp8(_) => AddressingMode::BxDisp8(displacement),
            AddressingMode::BxSiDisp16(_) => AddressingMode::BxSiDisp16(displacement),
            AddressingMode::BxDiDisp16(_) => AddressingMode::BxDiDisp16(displacement),
            AddressingMode::BpSiDisp16(_) => AddressingMode::BpSiDisp16(displacement),
            AddressingMode::BpDiDisp16(_) => AddressingMode::BpDiDisp16(displacement),
            AddressingMode::SiDisp16(_) => AddressingMode::SiDisp16(displacement),
            AddressingMode::DiDisp16(_) => AddressingMode::DiDisp16(displacement),
            AddressingMode::BpDisp16(_) => AddressingMode::BpDisp16(displacement),
            AddressingMode::BxDisp16(_) => AddressingMode::BxDisp16(displacement),
            _ => self.addressing_mode,
        }
    }
}
