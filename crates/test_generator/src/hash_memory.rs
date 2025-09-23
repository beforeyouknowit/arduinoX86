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
use std::collections::HashMap;

pub struct HashMemory {
    pub memory: HashMap<u32, u8>,
    pub uninit_byte: u8,
}

impl HashMemory {
    pub fn new() -> Self {
        HashMemory {
            memory: HashMap::new(),
            uninit_byte: 0,
        }
    }
    pub fn read_u8(&self, addr: u32) -> u8 {
        *self.memory.get(&addr).unwrap_or(&self.uninit_byte)
    }
    pub fn write_u8(&mut self, addr: u32, value: u8) {
        self.memory.insert(addr, value);
    }
    pub fn read_u16(&self, addr: u32) -> u16 {
        let low = self.read_u8(addr) as u16;
        let high = self.read_u8(addr + 1) as u16;
        (high << 8) | low
    }
    pub fn write_u16(&mut self, addr: u32, value: u16) {
        self.write_u8(addr, (value & 0xFF) as u8);
        self.write_u8(addr + 1, (value >> 8) as u8);
    }
    pub fn read_u32(&self, addr: u32) -> u32 {
        let b0 = self.read_u8(addr) as u32;
        let b1 = self.read_u8(addr + 1) as u32;
        let b2 = self.read_u8(addr + 2) as u32;
        let b3 = self.read_u8(addr + 3) as u32;
        (b3 << 24) | (b2 << 16) | (b1 << 8) | b0
    }
    pub fn write_u32(&mut self, addr: u32, value: u32) {
        self.write_u8(addr, (value & 0xFF) as u8);
        self.write_u8(addr + 1, ((value >> 8) & 0xFF) as u8);
        self.write_u8(addr + 2, ((value >> 16) & 0xFF) as u8);
        self.write_u8(addr + 3, ((value >> 24) & 0xFF) as u8);
    }

    pub fn read_bus16(&self, addr: u32) -> u16 {
        let addr16 = addr & !0x01;
        let low = self.read_u8(addr16) as u16;
        let high = self.read_u8(addr16 + 1) as u16;
        (high << 8) | low
    }

    pub fn write_bus16(&mut self, addr: u32, bhe: bool, value: u16) {
        let a0 = (addr & 0x01) != 0;
        let addr16 = addr & !0x01;

        let mut word = self.read_u16(addr16);
        if a0 && bhe {
            // Write high byte only
            word = (word & 0x00FF) | (value & 0xFF00);
        }
        else if !a0 && bhe {
            // Write full word
            word = value;
        }
        else if !a0 && !bhe {
            // Write low byte only
            word = (word & 0xFF00) | (value & 0x00FF);
        }
        else {
            // a0 == 1 && bhe == 0: refresh cycle
            return;
        }

        self.write_u16(addr16, word);
    }

    pub fn clear(&mut self) {
        self.memory.clear();
    }
    pub fn set_uninit_byte(&mut self, value: u8) {
        self.uninit_byte = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_bus16_aligned() {
        let mut mem = HashMemory::new();
        mem.write_u8(0x100, 0x34);
        mem.write_u8(0x101, 0x12);
        assert_eq!(mem.read_bus16(0x100), 0x1234);
    }

    #[test]
    fn test_read_bus16_unaligned() {
        let mut mem = HashMemory::new();
        mem.write_u8(0x100, 0x78);
        mem.write_u8(0x101, 0x56);
        // Should mask to 0x100
        assert_eq!(mem.read_bus16(0x101), 0x5678);
    }

    #[test]
    fn test_write_bus16_full_word() {
        let mut mem = HashMemory::new();
        mem.write_bus16(0x200, true, 0xBEEF);
        assert_eq!(mem.read_u16(0x200), 0xBEEF);
    }

    #[test]
    fn test_bus_readback_aligned() {
        let mut mem = HashMemory::new();
        mem.write_bus16(0x200, true, 0xBEEF);
        assert_eq!(mem.read_bus16(0x200), 0xBEEF);
    }

    #[test]
    fn test_bus_readback_unaligned1() {
        let mut mem = HashMemory::new();
        mem.write_bus16(0x200, true, 0xBEEF);
        assert_eq!(mem.read_bus16(0x201), 0xBEEF);
    }

    #[test]
    fn test_bus_readback_unaligned2() {
        let mut mem = HashMemory::new();
        mem.write_bus16(0x07CC0F, true, 0x5700);
        assert_eq!(mem.read_bus16(0x07CC0F), 0x5700);
    }
}
