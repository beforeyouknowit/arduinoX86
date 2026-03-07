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

use crate::enums::{AddressSize, InstructionSize};

use arduinox86_client::registers_common::SegmentSize;
use marty_dasm::Opcode;
use marty_isadb::IsaDB;
use moo::types::MooCpuType;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TestOpcodeSizePrefix {
    None,
    OperandSize,
    AddressSize,
    OperandAndAddressSize,
}

impl TestOpcodeSizePrefix {
    pub fn to_filename_prefix(&self) -> &'static str {
        match self {
            TestOpcodeSizePrefix::None => "",
            TestOpcodeSizePrefix::OperandSize => "66",
            TestOpcodeSizePrefix::AddressSize => "67",
            TestOpcodeSizePrefix::OperandAndAddressSize => "6766",
        }
    }

    /// Returns an iterator over all valid prefixes for the given CPU.
    pub fn iter(
        cpu_type: MooCpuType,
        opcode: Opcode,
        isa_db: &IsaDB,
        disable_operand_size_opcodes: &[u16],
        disable_address_size_opcodes: &[u16],
    ) -> Box<dyn Iterator<Item = TestOpcodeSizePrefix>> {
        match cpu_type {
            MooCpuType::Intel80386Ex => {
                let mut iter_vec = vec![TestOpcodeSizePrefix::None];

                let opcode_u16: u16 = opcode.into();

                let isa_entry = isa_db.opcode(opcode).expect("ISA DB opcode lookup failed");

                let use_operand_size =
                    isa_entry.can_use_operand_size_prefix && !disable_operand_size_opcodes.contains(&opcode_u16);
                let use_address_size =
                    isa_entry.can_use_address_size_prefix && !disable_address_size_opcodes.contains(&opcode_u16);

                if use_operand_size {
                    iter_vec.push(TestOpcodeSizePrefix::OperandSize);
                }
                if use_address_size {
                    iter_vec.push(TestOpcodeSizePrefix::AddressSize);
                }
                if use_operand_size && use_address_size {
                    iter_vec.push(TestOpcodeSizePrefix::OperandAndAddressSize);
                }
                Box::new(iter_vec.into_iter())
            }
            _ => Box::new(std::iter::empty()),
        }
    }

    pub fn relative_opcode_size(&self, size: SegmentSize) -> InstructionSize {
        match size {
            SegmentSize::Sixteen => match self {
                TestOpcodeSizePrefix::None => InstructionSize::Sixteen,
                TestOpcodeSizePrefix::OperandSize => InstructionSize::ThirtyTwo,
                TestOpcodeSizePrefix::AddressSize => InstructionSize::Sixteen,
                TestOpcodeSizePrefix::OperandAndAddressSize => InstructionSize::ThirtyTwo,
            },
            SegmentSize::ThirtyTwo => match self {
                TestOpcodeSizePrefix::None => InstructionSize::ThirtyTwo,
                TestOpcodeSizePrefix::OperandSize => InstructionSize::Sixteen,
                TestOpcodeSizePrefix::AddressSize => InstructionSize::ThirtyTwo,
                TestOpcodeSizePrefix::OperandAndAddressSize => InstructionSize::Sixteen,
            },
        }
    }

    pub fn relative_address_size(&self, size: SegmentSize) -> AddressSize {
        match size {
            SegmentSize::Sixteen => match self {
                TestOpcodeSizePrefix::None => AddressSize::Sixteen,
                TestOpcodeSizePrefix::OperandSize => AddressSize::Sixteen,
                TestOpcodeSizePrefix::AddressSize => AddressSize::ThirtyTwo,
                TestOpcodeSizePrefix::OperandAndAddressSize => AddressSize::ThirtyTwo,
            },
            SegmentSize::ThirtyTwo => match self {
                TestOpcodeSizePrefix::None => AddressSize::ThirtyTwo,
                TestOpcodeSizePrefix::OperandSize => AddressSize::ThirtyTwo,
                TestOpcodeSizePrefix::AddressSize => AddressSize::Sixteen,
                TestOpcodeSizePrefix::OperandAndAddressSize => AddressSize::Sixteen,
            },
        }
    }
}

impl From<TestOpcodeSizePrefix> for Vec<u8> {
    fn from(prefix: TestOpcodeSizePrefix) -> Self {
        match prefix {
            TestOpcodeSizePrefix::None => vec![],
            TestOpcodeSizePrefix::OperandSize => vec![0x66],
            TestOpcodeSizePrefix::AddressSize => vec![0x67],
            TestOpcodeSizePrefix::OperandAndAddressSize => vec![0x66, 0x67],
        }
    }
}
