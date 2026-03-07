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
    generate::gen_regs::TestRegisters,
    global_trace_log,
    instruction::instruction::TestInstruction,
    registers::Registers,
    trace_banner,
    trace_log,
    TestContext,
};
use marty_dasm::Opcode;
use moo::{prelude::*, registers::*};

impl TestInstruction {
    pub fn log_instruction(
        &self,
        ctx: &mut TestContext,
        test_num: usize,
        opcode: Opcode,
        op_ext: Option<u8>,
        test_registers: &TestRegisters,
    ) {
        let mut op_ext_str = String::new();
        if let Some(ext) = op_ext {
            // If this is a group opcode, append the extension.
            op_ext_str = format!(".{:1X}", ext);
        }

        let instruction_log_string = format!(
            "{:05} | {} | {:04X}:{:04X} | {}{} {:<35} │ {:02X?}",
            test_num,
            progress_string(ctx.gen_ct, ctx.gen_total),
            test_registers.regs.cs(),
            test_registers.regs.ip(),
            opcode,
            op_ext_str,
            self.name(),
            self.instr_bytes(),
        );

        if ctx.cfg.test_exec.print_instruction {
            println!("{}", instruction_log_string);
        }

        trace_banner!(ctx);
        global_trace_log!(ctx, ">>> {} test {}", ctx.exec_mode.gerund(), instruction_log_string);
        trace_log!(ctx, ">>> Op1:{:?} Op2:{:?}", self.op0_kind(), self.op1_kind());
        trace_banner!(ctx);

        // trace_log!(
        //     ctx,
        //     "Sequence bytes: {:02X?}",
        //     test_instruction.sequence_bytes()
        // );

        match &test_registers.regs {
            Registers::V2(regs) => {
                let moo_registers =
                    MooRegisters16::try_from(regs).expect("Failed to convert registers to MooRegisters");

                trace_log!(
                    ctx,
                    "{}",
                    MooRegisters16Printer {
                        regs: &moo_registers,
                        cpu_type: ctx.cfg.test_gen.cpu_type,
                        diff: None,
                        indent: 0,
                    }
                );
            }
            Registers::V3A(regs) => {
                let moo_registers =
                    MooRegisters32::try_from(regs).expect("Failed to convert registers to MooRegisters");

                trace_log!(
                    ctx,
                    "{}",
                    MooRegisters32Printer {
                        regs: &moo_registers,
                        cpu_type: ctx.cfg.test_gen.cpu_type,
                        diff: None,
                        indent: 0,
                    }
                );
            }
            Registers::V3B(regs) => {
                let moo_registers =
                    MooRegisters32::try_from(regs).expect("Failed to convert registers to MooRegisters");

                trace_log!(
                    ctx,
                    "{}",
                    MooRegisters32Printer {
                        regs: &moo_registers,
                        cpu_type: ctx.cfg.test_gen.cpu_type,
                        diff: None,
                        indent: 0,
                    }
                );
            }
            _ => {
                unimplemented!("Unsupported register set type for logging: {:?}", test_registers.regs);
            }
        }
    }
}

pub fn progress_string(test_num: usize, test_count: usize) -> String {
    if test_count == 0 {
        return "??%".to_string();
    }
    let percent_complete = (test_num as f64 / test_count as f64) * 100.0;
    format!("{:2.0}%", percent_complete)
}
