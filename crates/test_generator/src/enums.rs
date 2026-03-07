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
use serde::Deserialize;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub enum ExecMode {
    #[default]
    Generate,
    Validate,
}

impl ExecMode {
    pub fn gerund(&self) -> &'static str {
        match self {
            ExecMode::Generate => "Generating",
            ExecMode::Validate => "Validating",
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub enum InstructionSize {
    #[default]
    Sixteen,
    ThirtyTwo,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub enum AddressSize {
    #[default]
    Sixteen,
    ThirtyTwo,
}

impl From<InstructionSize> for u32 {
    fn from(size: InstructionSize) -> Self {
        match size {
            InstructionSize::Sixteen => 16,
            InstructionSize::ThirtyTwo => 32,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum CpuMode {
    Real,
    Unreal,
    Protected,
}

impl CpuMode {
    pub fn to_path_suffix(&self) -> &'static str {
        match self {
            CpuMode::Real => "real",
            CpuMode::Unreal => "unreal",
            CpuMode::Protected => "protected",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum TerminationCondition {
    Queue,
    Halt,
}
