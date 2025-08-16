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

#[derive(strum_macros::Display, Debug)]
pub enum CpuStateType {
    Initial,
    Final,
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
    FS,
    GS,
    CS,
    SS,
    DS,
    PC,
    InvalidRegister,
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
    EIP,
    InvalidRegister,
}

pub enum FileOpenContext {
    LoadWorkspace,
    LoadProgramSource,
    LoadProgramBinary,
    LoadRegisterBinary,
}

pub enum FileSaveContext {
    SaveWorkspace,
    SaveProgramSource,
    SaveProgramBinary,
    SaveRegisterBinary,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum BinaryBlobType {
    Program,
    Data,
    Registers,
}

#[derive(Copy, Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum MountAddress {
    CsIp,
    FlatAddress(u32),
}

impl MountAddress {
    pub fn flat_address(&self) -> Option<u32> {
        match self {
            MountAddress::CsIp => None,
            MountAddress::FlatAddress(addr) => Some(*addr),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ClientControlState {
    #[default]
    Setup,
    Running,
    Error,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScheduleType {
    OneShot,
    Repeat,
}
