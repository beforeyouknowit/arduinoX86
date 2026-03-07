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

#[derive(Default)]
pub struct InstructionName {
    resolved: String,
    iced: String,
    marty_dasm: String,
}

impl InstructionName {
    pub fn resolved(&self) -> &str {
        &self.resolved
    }

    pub fn from_resolved(name: &str) -> Self {
        InstructionName {
            resolved: name.to_string(),
            iced: String::new(),
            marty_dasm: String::new(),
        }
    }

    pub fn iced(&self) -> &str {
        &self.iced
    }

    pub fn set_iced(&mut self, name: &str) {
        self.iced = name.to_string();
    }

    pub fn from_iced(name: &str) -> Self {
        InstructionName {
            resolved: name.to_string(),
            iced: name.to_string(),
            marty_dasm: String::new(),
        }
    }

    pub fn from_marty(name: &str) -> Self {
        InstructionName {
            resolved: name.to_string(),
            iced: String::new(),
            marty_dasm: name.to_string(),
        }
    }
}
