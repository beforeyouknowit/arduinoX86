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
pub const CPU_FLAG_CARRY: u16 = 0b0000_0000_0000_0001;
pub const CPU_FLAG_RESERVED1: u16 = 0b0000_0000_0000_0010;
pub const CPU_FLAG_PARITY: u16 = 0b0000_0000_0000_0100;
pub const CPU_FLAG_RESERVED3: u16 = 0b0000_0000_0000_1000;
pub const CPU_FLAG_AUX_CARRY: u16 = 0b0000_0000_0001_0000;
pub const CPU_FLAG_RESERVED5: u16 = 0b0000_0000_0010_0000;
pub const CPU_FLAG_ZERO: u16 = 0b0000_0000_0100_0000;
pub const CPU_FLAG_SIGN: u16 = 0b0000_0000_1000_0000;
pub const CPU_FLAG_TRAP: u16 = 0b0000_0001_0000_0000;
pub const CPU_FLAG_INT_ENABLE: u16 = 0b0000_0010_0000_0000;
pub const CPU_FLAG_DIRECTION: u16 = 0b0000_0100_0000_0000;
pub const CPU_FLAG_OVERFLOW: u16 = 0b0000_1000_0000_0000;
pub const CPU_FLAG_F15: u16 = 0b1000_0000_0000_0000; // Reserved bit 15
pub const CPU_FLAG_MODE: u16 = 0b1000_0000_0000_0000;
pub const CPU_FLAG_NT: u16 = 0b0100_0000_0000_0000; // Nested Task
pub const CPU_FLAG_IOPL0: u16 = 0b0001_0000_0000_0000; // Nested Task
pub const CPU_FLAG_IOPL1: u16 = 0b0010_0000_0000_0000; // Nested Task
