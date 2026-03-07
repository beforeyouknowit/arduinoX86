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
use crate::{cpu_common::Displacement, registers::Registers, TestContext};

use iced_x86::{Decoder, Formatter, NasmFormatter};

#[derive(Default)]
pub struct IcedDisassembly {
    pub(crate) iced_i: iced_x86::Instruction,
    pub(crate) disp: Option<Displacement>,
    pub(crate) disp_offset: usize,
}

impl IcedDisassembly {
    pub fn disassemble(context: &mut TestContext, bytes: &[u8], regs: &Registers) -> IcedDisassembly {
        let mut iced_decoder = Decoder::new(context.code_segment_size.into(), bytes, context.iced_decoder_opts);
        iced_decoder.set_ip(regs.eip().into());
        let iced_i = iced_decoder.decode();

        let (disp, disp_offset) = IcedDisassembly::get_displacement(&mut iced_decoder, &iced_i, bytes);

        IcedDisassembly {
            iced_i,
            disp,
            disp_offset,
        }
    }

    /// Return the disassembled instruction text and mnemonic string for the provided iced instruction.
    pub fn format(&self) -> (String, String) {
        let mut instr_text = String::new();
        let mut formatter = NasmFormatter::new();

        formatter.options_mut().set_always_show_segment_register(true);
        formatter.options_mut().set_add_leading_zero_to_hex_numbers(false);
        formatter.options_mut().set_always_show_scale(true);
        //formatter.options_mut().set_show_zero_displacements(true);

        formatter.format(&self.iced_i, &mut instr_text);

        let mut mnemonic_string = String::new();
        formatter.format_mnemonic_options(
            &self.iced_i,
            &mut mnemonic_string,
            iced_x86::FormatMnemonicOptions::NO_PREFIXES,
        );

        // Remove spurious 'notrack' extension decoding.
        instr_text = instr_text.replace("notrack ", "");

        (instr_text, mnemonic_string)
    }

    pub fn get_displacement(
        decoder: &mut Decoder,
        instruction: &iced_x86::Instruction,
        bytes: &[u8],
    ) -> (Option<Displacement>, usize) {
        let constant_offsets = decoder.get_constant_offsets(&instruction);

        let d_offset = constant_offsets.displacement_offset();

        if d_offset >= bytes.len() {
            log::error!(
                "get_displacement(): Displacement offset {} is out of bounds for instruction bytes {:X?}",
                d_offset,
                bytes
            );
            return (None, 0);
        }

        match constant_offsets.displacement_size() {
            1 => {
                log::trace!("get_displacement(): Getting 8-bit displacement...");
                (Some(Displacement::Disp8(bytes[d_offset] as i8)), d_offset)
            }
            2 => {
                log::trace!("get_displacement(): Getting 16-bit displacement...");
                let disp_bytes = &bytes[d_offset..(d_offset + 2)];
                (
                    Some(Displacement::Disp16(
                        u16::from_le_bytes(disp_bytes.try_into().unwrap()) as i16
                    )),
                    d_offset,
                )
            }
            4 => {
                let disp_bytes = &bytes[d_offset..(d_offset + 4)];
                let disp = u32::from_le_bytes(disp_bytes.try_into().unwrap());
                log::trace!(
                    "get_displacement(): Getting 32-bit displacement from bytes {:X?} {:04X}",
                    disp_bytes,
                    disp
                );
                (
                    Some(Displacement::Disp32(
                        u32::from_le_bytes(disp_bytes.try_into().unwrap()) as i32
                    )),
                    d_offset,
                )
            }
            _ => {
                log::trace!("get_displacement(): No displacement.");
                (None, 0)
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.iced_i.code() != iced_x86::Code::INVALID
    }

    pub fn len(&self) -> usize {
        self.iced_i.len()
    }

    pub fn i(&self) -> &iced_x86::Instruction {
        &self.iced_i
    }

    pub fn i_mut(&mut self) -> &mut iced_x86::Instruction {
        &mut self.iced_i
    }

    pub fn disp(&self) -> Option<Displacement> {
        self.disp
    }
}
