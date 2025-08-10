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
use crate::{
    registers::register_traits::Registers32,
    registers_v3::RemoteCpuRegistersV3,
    RemoteCpuRegisters,
    RemoteCpuRegistersV1,
    RemoteCpuRegistersV2,
    ServerCpuType,
};
use std::fmt::Display;

#[macro_export]
macro_rules! flag_chr {
    ($flags:expr, $flag:expr, $yes:expr, $no:expr) => {
        if ($flag & $flags) != 0 {
            $yes
        }
        else {
            $no
        }
    };
}

pub struct RegisterPrinter<'a> {
    pub regs: &'a RemoteCpuRegisters,
    pub final_regs: Option<&'a RemoteCpuRegisters>,
    pub cpu_type: ServerCpuType,
    pub options: u32,
}

impl RegisterPrinter<'_> {
    pub const OPTION_NO_XREGS: u32 = 0x0001;
    pub const NO_TR: u32 = 0x0002;
    pub const NO_LDT: u32 = 0x0004;
}

impl Display for RegisterPrinter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.regs, &self.final_regs) {
            (RemoteCpuRegisters::V1(regs), None) => fmt_regs_v1(f, regs, self.cpu_type),
            (RemoteCpuRegisters::V2(regs), None) => fmt_regs_v2(f, regs, self.cpu_type, self.options),
            (RemoteCpuRegisters::V3(regs), None) => fmt_regs_v3(f, regs, self.cpu_type),
            (RemoteCpuRegisters::V1(regs), Some(RemoteCpuRegisters::V1(final_regs))) => {
                fmt_regs_v1_delta(f, regs, final_regs, self.cpu_type)
            }
            (RemoteCpuRegisters::V2(regs), Some(RemoteCpuRegisters::V2(final_regs))) => {
                fmt_regs_v2_delta(f, regs, final_regs, self.cpu_type, self.options)
            }
            (RemoteCpuRegisters::V3(regs), Some(RemoteCpuRegisters::V3(final_regs))) => {
                fmt_regs_v3_delta(f, regs, final_regs, self.options)
            }
            _ => Ok(()),
        }
    }
}

pub fn fmt_regs_v1(
    f: &mut std::fmt::Formatter<'_>,
    regs: &RemoteCpuRegistersV1,
    cpu_type: ServerCpuType,
) -> std::fmt::Result {
    let reg_str = format!(
        "AX: {:04X} BX: {:04X} CX: {:04X} DX: {:04X}\n\
         SI: {:04X} DI: {:04X} BP: {:04X} SP: {:04X}\n\
         CS: {:04X} DS: {:04X} ES: {:04X} SS: {:04X}\n\
         IP: {:04X}\n\
         FLAGS: {:04X}",
        regs.ax,
        regs.bx,
        regs.cx,
        regs.dx,
        regs.si,
        regs.di,
        regs.bp,
        regs.sp,
        regs.cs,
        regs.ds,
        regs.es,
        regs.ss,
        regs.ip,
        regs.flags
    );

    write!(f, "{} ", reg_str)?;

    // Expand flag info
    fmt_flags_v1(f, regs.flags, cpu_type)
}

pub fn fmt_regs_v2(
    f: &mut std::fmt::Formatter<'_>,
    regs: &RemoteCpuRegistersV2,
    cpu_type: ServerCpuType,
    options: u32,
) -> std::fmt::Result {
    if options & RegisterPrinter::OPTION_NO_XREGS == 0 {
        write!(
            f,
            "X0: {:04X} X1: {:04X} X2: {:04X} X3: {:04X} X4: {:04X}\n\
             X5: {:04X} X6: {:04X} X7: {:04X} X8: {:04X} X9: {:04X}\n",
            regs.x0, regs.x1, regs.x2, regs.x3, regs.x4, regs.x5, regs.x6, regs.x7, regs.x8, regs.x9
        )?;
    }

    let v1_regs = RemoteCpuRegistersV1::from(regs);

    write!(f, "MSW: {:04X}\n", regs.msw)?;

    if options & RegisterPrinter::NO_TR == 0 {
        write!(f, " TR: {:04X}", regs.tr)?;
    }

    if options & RegisterPrinter::NO_LDT == 0 {
        write!(f, " LDT: {:04X}\n", regs.ldt)?;
    }

    fmt_regs_v1(f, &v1_regs, cpu_type)
}

pub fn fmt_regs_v3(
    fmt: &mut std::fmt::Formatter<'_>,
    regs: &RemoteCpuRegistersV3,
    cpu_type: ServerCpuType,
) -> std::fmt::Result {
    let reg_str = format!(
        "EAX: {:08X} EBX: {:08X} ECX: {:08X} EDX: {:08X}\n\
         ESI: {:08X} EDI: {:08X} EBP: {:08X} ESP: {:08X}\n\
         CS: {:04X} DS: {:04X} ES: {:04X} FS: {:04X} GS: {:04X} SS: {:04X}\n\
         EIP: {:08X}\n\
         FLAGS: {:08X}",
        regs.eax(),
        regs.ebx(),
        regs.ecx(),
        regs.edx(),
        regs.esi(),
        regs.edi(),
        regs.ebp(),
        regs.esp(),
        regs.cs(),
        regs.ds(),
        regs.es(),
        regs.fs(),
        regs.gs(),
        regs.ss(),
        regs.eip(),
        regs.eflags()
    );

    write!(fmt, "{} ", reg_str)?;

    // Expand flag info
    fmt_flags_v3(fmt, regs.eflags())
}

/// Format the flags for 16-bit CPUs.
/// This function takes a [ServerCpuType] so that it can interpret certain flags that are defined
/// differently per CPU (mode flag for V20, NT and IOPL flags for 80286).
pub fn fmt_flags_v1(fmt: &mut std::fmt::Formatter<'_>, flags: u16, cpu_type: ServerCpuType) -> std::fmt::Result {
    let f = flags;
    let c_chr = flag_chr!(f, RemoteCpuRegistersV1::FLAG_CARRY, 'C', 'c');
    let p_chr = flag_chr!(f, RemoteCpuRegistersV1::FLAG_PARITY, 'P', 'p');
    let a_chr = flag_chr!(f, RemoteCpuRegistersV1::FLAG_AUX_CARRY, 'A', 'a');
    let z_chr = flag_chr!(f, RemoteCpuRegistersV1::FLAG_ZERO, 'Z', 'z');
    let s_chr = flag_chr!(f, RemoteCpuRegistersV1::FLAG_SIGN, 'S', 's');
    let t_chr = flag_chr!(f, RemoteCpuRegistersV1::FLAG_TRAP, 'T', 't');
    let i_chr = flag_chr!(f, RemoteCpuRegistersV1::FLAG_INT_ENABLE, 'I', 'i');
    let d_chr = flag_chr!(f, RemoteCpuRegistersV1::FLAG_DIRECTION, 'D', 'd');
    let o_chr = flag_chr!(f, RemoteCpuRegistersV1::FLAG_OVERFLOW, 'O', 'o');
    let m_chr = if cpu_type.is_intel() {
        if matches!(cpu_type, ServerCpuType::Intel80286) {
            flag_chr!(f, RemoteCpuRegistersV1::FLAG_F15, '1', '0')
        }
        else {
            '1'
        }
    }
    else {
        flag_chr!(f, RemoteCpuRegistersV1::FLAG_MODE, 'M', 'm')
    };

    let nt_chr = if matches!(cpu_type, ServerCpuType::Intel80286) {
        flag_chr!(f, RemoteCpuRegistersV1::FLAG_NT, '1', '0')
    }
    else {
        '1'
    };

    let iopl0_chr = flag_chr!(f, RemoteCpuRegistersV1::FLAG_IOPL0, '0', '1');
    let iopl1_chr = flag_chr!(f, RemoteCpuRegistersV1::FLAG_IOPL1, '0', '1');

    write!(
        fmt,
        "{}{}{}{}{}{}{}{}{}{}0{}0{}1{}",
        m_chr, nt_chr, iopl1_chr, iopl0_chr, o_chr, d_chr, i_chr, t_chr, s_chr, z_chr, a_chr, p_chr, c_chr
    )
}

/// Format flags for 32-bit CPUs.
pub fn fmt_flags_v3(fmt: &mut std::fmt::Formatter<'_>, flags: u32) -> std::fmt::Result {
    let f = flags;
    let c_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_CARRY, 'C', 'c');
    let p_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_PARITY, 'P', 'p');
    let a_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_AUX_CARRY, 'A', 'a');
    let z_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_ZERO, 'Z', 'z');
    let s_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_SIGN, 'S', 's');
    let t_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_TRAP, 'T', 't');
    let i_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_INT_ENABLE, 'I', 'i');
    let d_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_DIRECTION, 'D', 'd');
    let o_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_OVERFLOW, 'O', 'o');
    let m_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_F15, '1', '0');
    let nt_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_NT, '1', '0');
    let iopl0_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_IOPL0, '0', '1');
    let iopl1_chr = flag_chr!(f, RemoteCpuRegistersV3::FLAG_IOPL1, '0', '1');

    write!(
        fmt,
        "{}{}{}{}{}{}{}{}{}{}0{}0{}1{}",
        m_chr, nt_chr, iopl1_chr, iopl0_chr, o_chr, d_chr, i_chr, t_chr, s_chr, z_chr, a_chr, p_chr, c_chr
    )
}

pub fn fmt_regs_v1_delta(
    fmt: &mut std::fmt::Formatter<'_>,
    initial: &RemoteCpuRegistersV1,
    _final: &RemoteCpuRegistersV1,
    cpu_type: ServerCpuType,
) -> std::fmt::Result {
    let a_diff = initial.ax != _final.ax;
    let b_diff = initial.bx != _final.bx;
    let c_diff = initial.cx != _final.cx;
    let d_diff = initial.dx != _final.dx;
    let sp_diff = initial.sp != _final.sp;
    let bp_diff = initial.bp != _final.bp;
    let si_diff = initial.si != _final.si;
    let di_diff = initial.di != _final.di;
    let cs_diff = initial.cs != _final.cs;
    let ds_diff = initial.ds != _final.ds;
    let es_diff = initial.es != _final.es;
    let ss_diff = initial.ss != _final.ss;
    let ip_diff = initial.ip != _final.ip;
    let f_diff = initial.flags != _final.flags;

    let reg_str = format!(
        "AX:{}{:04X} BX:{}{:04X} CX:{}{:04X} DX:{}{:04X}\n\
         SP:{}{:04X} BP:{}{:04X} SI:{}{:04X} DI:{}{:04X}\n\
         CS:{}{:04X} DS:{}{:04X} ES:{}{:04X} SS:{}{:04X}\n\
         IP: {:04X}\n\
         FLAGS:{}{:04X}",
        if a_diff { "*" } else { " " },
        _final.ax,
        if b_diff { "*" } else { " " },
        _final.bx,
        if c_diff { "*" } else { " " },
        _final.cx,
        if d_diff { "*" } else { " " },
        _final.dx,
        if sp_diff { "*" } else { " " },
        _final.sp,
        if bp_diff { "*" } else { " " },
        _final.bp,
        if si_diff { "*" } else { " " },
        _final.si,
        if di_diff { "*" } else { " " },
        _final.di,
        if cs_diff { "*" } else { " " },
        _final.cs,
        if ds_diff { "*" } else { " " },
        _final.ds,
        if es_diff { "*" } else { " " },
        _final.es,
        if ss_diff { "*" } else { " " },
        _final.ss,
        _final.ip,
        if f_diff { "*" } else { " " },
        _final.flags
    );

    write!(fmt, "{} ", reg_str)?;
    fmt_flags_v1(fmt, _final.flags, cpu_type)
}

pub fn fmt_regs_v2_delta(
    fmt: &mut std::fmt::Formatter<'_>,
    initial: &RemoteCpuRegistersV2,
    _final: &RemoteCpuRegistersV2,
    cpu_type: ServerCpuType,
    options: u32,
) -> std::fmt::Result {
    if options & RegisterPrinter::OPTION_NO_XREGS == 0 {
        let x0_diff = initial.x0 != _final.x0;
        let x1_diff = initial.x1 != _final.x1;
        let x2_diff = initial.x2 != _final.x2;
        let x3_diff = initial.x3 != _final.x3;
        let x4_diff = initial.x4 != _final.x4;
        let x5_diff = initial.x5 != _final.x5;
        let x6_diff = initial.x6 != _final.x6;
        let x7_diff = initial.x7 != _final.x7;
        let x8_diff = initial.x8 != _final.x8;
        let x9_diff = initial.x9 != _final.x9;

        write!(
            fmt,
            "X0:{}{:04X} X1:{}{:04X} X2:{}{:04X} X3:{}{:04X} X4:{}{:04X}\n\
             X5:{}{:04X} X6:{}{:04X} X7:{}{:04X} X8:{}{:04X} X9:{}{:04X}\n",
            if x0_diff { "*" } else { " " },
            _final.x0,
            if x1_diff { "*" } else { " " },
            _final.x1,
            if x2_diff { "*" } else { " " },
            _final.x2,
            if x3_diff { "*" } else { " " },
            _final.x3,
            if x4_diff { "*" } else { " " },
            _final.x4,
            if x5_diff { "*" } else { " " },
            _final.x5,
            if x6_diff { "*" } else { " " },
            _final.x6,
            if x7_diff { "*" } else { " " },
            _final.x7,
            if x8_diff { "*" } else { " " },
            _final.x8,
            if x9_diff { "*" } else { " " },
            _final.x9
        )?;
    }

    let a_diff = initial.ax != _final.ax;
    let b_diff = initial.bx != _final.bx;
    let c_diff = initial.cx != _final.cx;
    let d_diff = initial.dx != _final.dx;
    let sp_diff = initial.sp != _final.sp;
    let bp_diff = initial.bp != _final.bp;
    let si_diff = initial.si != _final.si;
    let di_diff = initial.di != _final.di;
    let cs_diff = initial.cs != _final.cs;
    let ds_diff = initial.ds != _final.ds;
    let es_diff = initial.es != _final.es;
    let ss_diff = initial.ss != _final.ss;
    let ip_diff = initial.ip != _final.ip;
    let f_diff = initial.flags != _final.flags;

    let reg_str = format!(
        "AX:{}{:04X} BX:{}{:04X} CX:{}{:04X} DX:{}{:04X}\n\
         SP:{}{:04X} BP:{}{:04X} SI:{}{:04X} DI:{}{:04X}\n\
         CS:{}{:04X} DS:{}{:04X} ES:{}{:04X} SS:{}{:04X}\n\
         IP: {:04X}\n\
         FLAGS:{}{:04X}",
        if a_diff { "*" } else { " " },
        _final.ax,
        if b_diff { "*" } else { " " },
        _final.bx,
        if c_diff { "*" } else { " " },
        _final.cx,
        if d_diff { "*" } else { " " },
        _final.dx,
        if sp_diff { "*" } else { " " },
        _final.sp,
        if bp_diff { "*" } else { " " },
        _final.bp,
        if si_diff { "*" } else { " " },
        _final.si,
        if di_diff { "*" } else { " " },
        _final.di,
        if cs_diff { "*" } else { " " },
        _final.cs,
        if ds_diff { "*" } else { " " },
        _final.ds,
        if es_diff { "*" } else { " " },
        _final.es,
        if ss_diff { "*" } else { " " },
        _final.ss,
        _final.ip,
        if f_diff { "*" } else { " " },
        _final.flags
    );

    write!(fmt, "{} ", reg_str)?;
    fmt_flags_v1(fmt, _final.flags, cpu_type)
}

pub fn fmt_regs_v3_delta(
    fmt: &mut std::fmt::Formatter<'_>,
    initial: &RemoteCpuRegistersV3,
    regs: &RemoteCpuRegistersV3,
    _options: u32,
) -> std::fmt::Result {
    let cr0_diff = initial.cr0() != regs.cr0();
    let a_diff = initial.eax() != regs.eax();
    let b_diff = initial.ebx() != regs.ebx();
    let c_diff = initial.ecx() != regs.ecx();
    let d_diff = initial.edx() != regs.edx();
    let sp_diff = initial.esp() != regs.esp();
    let bp_diff = initial.ebp() != regs.ebp();
    let si_diff = initial.esi() != regs.esi();
    let di_diff = initial.edi() != regs.edi();
    let cs_diff = initial.cs() != regs.cs();
    let ds_diff = initial.ds() != regs.ds();
    let es_diff = initial.es() != regs.es();
    let fs_diff = initial.fs() != regs.fs();
    let gs_diff = initial.gs() != regs.gs();
    let ss_diff = initial.ss() != regs.ss();
    let ip_diff = initial.eip() != regs.eip();
    let f_diff = initial.eflags() != regs.eflags();
    let dr6_diff = initial.dr6() != regs.dr6();
    let dr7_diff = initial.dr7() != regs.dr7();

    let reg_str = format!(
        "CR0:{}{:08X}\n\
         EAX:{}{:08X} EBX:{}{:08X} ECX:{}{:08X} EDX:{}{:08X}\n\
         ESI:{}{:08X} EDI:{}{:08X} EBP:{}{:08X} ESP:{}{:08X}\n\
         CS:{}{:04X} DS:{}{:04X} ES:{}{:04X} FS:{}{:04X} GS:{}{:04X} SS:{}{:04X}\n\
         EIP:{}{:08X} DR6:{}{:08X} DR7:{}{:08X}\n\
         EFLAGS:{}{:08X}",
        if cr0_diff { "*" } else { " " },
        regs.cr0(),
        if a_diff { "*" } else { " " },
        regs.eax(),
        if b_diff { "*" } else { " " },
        regs.ebx(),
        if c_diff { "*" } else { " " },
        regs.ecx(),
        if d_diff { "*" } else { " " },
        regs.edx(),
        if di_diff { "*" } else { " " },
        regs.esi(),
        if si_diff { "*" } else { " " },
        regs.edi(),
        if bp_diff { "*" } else { " " },
        regs.ebp(),
        if sp_diff { "*" } else { " " },
        regs.esp(),
        if cs_diff { "*" } else { " " },
        regs.cs(),
        if ds_diff { "*" } else { " " },
        regs.ds(),
        if es_diff { "*" } else { " " },
        regs.es(),
        if fs_diff { "*" } else { " " },
        regs.fs(),
        if gs_diff { "*" } else { " " },
        regs.gs(),
        if ss_diff { "*" } else { " " },
        regs.ss(),
        if ip_diff { "*" } else { " " },
        regs.eip(),
        if dr6_diff { "*" } else { " " },
        regs.dr6(),
        if dr7_diff { "*" } else { " " },
        regs.dr7(),
        if f_diff { "*" } else { " " },
        regs.eflags()
    );

    write!(fmt, "{} ", reg_str)?;

    // Expand flag info
    fmt_flags_v3(fmt, regs.eflags())
}
