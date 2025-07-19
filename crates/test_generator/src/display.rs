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

use crate::{flags::*, registers::Registers};
use arduinox86_client::{RemoteCpuRegistersV1, RemoteCpuRegistersV2, ServerCpuType};

pub fn print_regs_v1(regs: &RemoteCpuRegistersV1, cpu_type: ServerCpuType) {
    let reg_str = format!(
        "AX: {:04X} BX: {:04X} CX: {:04X} DX: {:04X}\n\
          SP: {:04X} BP: {:04X} SI: {:04X} DI: {:04X}\n\
          CS: {:04X} DS: {:04X} ES: {:04X} SS: {:04X}\n\
          IP: {:04X}\n\
          FLAGS: {:04X}",
        regs.ax,
        regs.bx,
        regs.cx,
        regs.dx,
        regs.sp,
        regs.bp,
        regs.si,
        regs.di,
        regs.cs,
        regs.ds,
        regs.es,
        regs.ss,
        regs.ip,
        regs.flags
    );

    print!("{} ", reg_str);

    // Expand flag info
    let f = regs.flags;
    let c_chr = if CPU_FLAG_CARRY & f != 0 { 'C' } else { 'c' };
    let p_chr = if CPU_FLAG_PARITY & f != 0 { 'P' } else { 'p' };
    let a_chr = if CPU_FLAG_AUX_CARRY & f != 0 { 'A' } else { 'a' };
    let z_chr = if CPU_FLAG_ZERO & f != 0 { 'Z' } else { 'z' };
    let s_chr = if CPU_FLAG_SIGN & f != 0 { 'S' } else { 's' };
    let t_chr = if CPU_FLAG_TRAP & f != 0 { 'T' } else { 't' };
    let i_chr = if CPU_FLAG_INT_ENABLE & f != 0 { 'I' } else { 'i' };
    let d_chr = if CPU_FLAG_DIRECTION & f != 0 { 'D' } else { 'd' };
    let o_chr = if CPU_FLAG_OVERFLOW & f != 0 { 'O' } else { 'o' };
    let m_chr = if cpu_type.is_intel() {
        if matches!(cpu_type, ServerCpuType::Intel80286) {
            if CPU_FLAG_F15 & f != 0 {
                '1'
            }
            else {
                '0'
            }
        }
        else {
            '1'
        }
    }
    else {
        if f & CPU_FLAG_MODE != 0 {
            'M'
        }
        else {
            'm'
        }
    };

    let nt_chr = if f & CPU_FLAG_NT != 0 { '1' } else { '0' };
    let iopl0_chr = if f & CPU_FLAG_IOPL0 != 0 { '1' } else { '0' };
    let iopl1_chr = if f & CPU_FLAG_IOPL1 != 0 { '1' } else { '0' };

    println!(
        "{}{}{}{}{}{}{}{}{}{}0{}0{}1{}",
        m_chr, nt_chr, iopl1_chr, iopl0_chr, o_chr, d_chr, i_chr, t_chr, s_chr, z_chr, a_chr, p_chr, c_chr
    );
}

pub fn print_regs_v2(regs: &RemoteCpuRegistersV2, cpu_type: ServerCpuType) {
    println!(
        "X0: {:04X} X1: {:04X} X2: {:04X} X3: {:04X} X4: {:04X}\n\
             X5: {:04X} X6: {:04X} X7: {:04X} X8: {:04X} X9: {:04X}",
        regs.x0, regs.x1, regs.x2, regs.x3, regs.x4, regs.x5, regs.x6, regs.x7, regs.x8, regs.x9
    );

    let v1_regs = RemoteCpuRegistersV1::from(regs);

    println!("MSW: {:04X} TR: {:04X} LDT: {:04X}", regs.msw, regs.tr, regs.ldt);

    print_regs_v1(&v1_regs, cpu_type);
}

pub fn print_regs(regs: &Registers, cpu_type: ServerCpuType) {
    match regs {
        Registers::V1(regs_v1) => {
            print_regs_v1(regs_v1, cpu_type);
        }
        Registers::V2(regs_v2) => {
            print_regs_v2(regs_v2, cpu_type);
        }
    }
}
