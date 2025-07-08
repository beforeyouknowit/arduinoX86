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

#[derive(Copy, Clone, Debug)]
pub enum Displacement {
    NoDisp,
    Pending8,
    Pending16,
    Disp8(i8),
    Disp16(i16),
}

#[derive(Copy, Clone, Debug)]
pub enum AddressingMode {
    BxSi,
    BxDi,
    BpSi,
    BpDi,
    Si,
    Di,
    Disp16(Displacement),
    Bx,
    BxSiDisp8(Displacement),
    BxDiDisp8(Displacement),
    BpSiDisp8(Displacement),
    BpDiDisp8(Displacement),
    SiDisp8(Displacement),
    DiDisp8(Displacement),
    BpDisp8(Displacement),
    BxDisp8(Displacement),
    BxSiDisp16(Displacement),
    BxDiDisp16(Displacement),
    BpSiDisp16(Displacement),
    BpDiDisp16(Displacement),
    SiDisp16(Displacement),
    DiDisp16(Displacement),
    BpDisp16(Displacement),
    BxDisp16(Displacement),
    RegisterMode,
    RegisterIndirect(Register16),
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
    PC,
    InvalidRegister,
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

pub const SREGISTER_LUT: [Register16; 8] = [
    Register16::ES,
    Register16::CS,
    Register16::SS,
    Register16::DS,
    Register16::ES,
    Register16::CS,
    Register16::SS,
    Register16::DS,
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
