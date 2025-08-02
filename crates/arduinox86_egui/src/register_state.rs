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
use arduinox86_client::{RemoteCpuRegistersV3, ServerCpuType};

#[derive(Debug, Clone)]
pub struct RegisterStringStateV3 {
    pub cr0: String,
    pub eflags_raw: String,
    pub eip: String,
    pub edi: String,
    pub esi: String,
    pub ebp: String,
    pub esp: String,
    pub ebx: String,
    pub edx: String,
    pub ecx: String,
    pub eax: String,
    pub dr6: String,
    pub dr7: String,
    pub gs: String,
    pub fs: String,
    pub ds: String,
    pub ss: String,
    pub cs: String,
    pub es: String,
    pub flags: FlagStringState,
}

impl Default for RegisterStringStateV3 {
    fn default() -> Self {
        RegisterStringStateV3 {
            cr0: "00000000".to_string(),
            eflags_raw: "00000000".to_string(),
            eip: "00000000".to_string(),
            edi: "00000000".to_string(),
            esi: "00000000".to_string(),
            ebp: "00000000".to_string(),
            esp: "00000000".to_string(),
            ebx: "00000000".to_string(),
            edx: "00000000".to_string(),
            ecx: "00000000".to_string(),
            eax: "00000000".to_string(),
            dr6: "00000000".to_string(),
            dr7: "00000000".to_string(),
            gs: "0000".to_string(),
            fs: "0000".to_string(),
            ds: "0000".to_string(),
            ss: "0000".to_string(),
            cs: "0000".to_string(),
            es: "0000".to_string(),
            flags: FlagStringState::default(),
        }
    }
}

impl From<&RemoteCpuRegistersV3> for RegisterStringStateV3 {
    fn from(regs: &RemoteCpuRegistersV3) -> Self {
        RegisterStringStateV3 {
            cr0: format!("{:#08x}", regs.cr0),
            eflags_raw: format!("{:#08x}", regs.eflags),
            eip: format!("{:08x}", regs.eip),
            edi: format!("{:08x}", regs.edi),
            esi: format!("{:08x}", regs.esi),
            ebp: format!("{:08x}", regs.ebp),
            esp: format!("{:08x}", regs.esp),
            ebx: format!("{:08x}", regs.ebx),
            edx: format!("{:08x}", regs.edx),
            ecx: format!("{:08x}", regs.ecx),
            eax: format!("{:08x}", regs.eax),
            dr6: format!("{:08x}", regs.dr6),
            dr7: format!("{:08x}", regs.dr7),
            gs: format!("{:04x}", regs.gs),
            fs: format!("{:04x}", regs.fs),
            ds: format!("{:04x}", regs.ds),
            ss: format!("{:04x}", regs.ss),
            cs: format!("{:04x}", regs.cs),
            es: format!("{:04x}", regs.es),
            flags: FlagStringState::new(regs.eflags, ServerCpuType::Intel80386),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlagStringState {
    pub c_fl: String,
    pub p_fl: String,
    pub a_fl: String,
    pub z_fl: String,
    pub s_fl: String,
    pub t_fl: String,
    pub i_fl: String,
    pub d_fl: String,
    pub o_fl: String,
    pub m_fl: String,
}

impl Default for FlagStringState {
    fn default() -> Self {
        FlagStringState {
            c_fl: "0".to_string(),
            p_fl: "0".to_string(),
            a_fl: "0".to_string(),
            z_fl: "0".to_string(),
            s_fl: "0".to_string(),
            t_fl: "0".to_string(),
            i_fl: "0".to_string(),
            d_fl: "0".to_string(),
            o_fl: "0".to_string(),
            m_fl: "1".to_string(),
        }
    }
}

impl FlagStringState {
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

    pub fn new(flags_raw: u32, cpu_type: ServerCpuType) -> Self {
        FlagStringState {
            c_fl: {
                let fl = flags_raw & Self::FLAG_CARRY > 0;
                format!("{:1}", fl as u8)
            },
            p_fl: {
                let fl = flags_raw & Self::FLAG_PARITY > 0;
                format!("{:1}", fl as u8)
            },
            a_fl: {
                let fl = flags_raw & Self::FLAG_AUX_CARRY > 0;
                format!("{:1}", fl as u8)
            },
            z_fl: {
                let fl = flags_raw & Self::FLAG_ZERO > 0;
                format!("{:1}", fl as u8)
            },
            s_fl: {
                let fl = flags_raw & Self::FLAG_SIGN > 0;
                format!("{:1}", fl as u8)
            },
            t_fl: {
                let fl = flags_raw & Self::FLAG_TRAP > 0;
                format!("{:1}", fl as u8)
            },
            i_fl: {
                let fl = flags_raw & Self::FLAG_INT_ENABLE > 0;
                format!("{:1}", fl as u8)
            },
            d_fl: {
                let fl = flags_raw & Self::FLAG_DIRECTION > 0;
                format!("{:1}", fl as u8)
            },
            o_fl: {
                let fl = flags_raw & Self::FLAG_OVERFLOW > 0;
                format!("{:1}", fl as u8)
            },
            m_fl: { "1".to_string() },
        }
    }
}
