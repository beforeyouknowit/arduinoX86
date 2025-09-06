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
    cpu_common::{BusOp, BusOpType},
    cycles::MyServerCycleState,
    registers::Registers,
    trace_log,
    Config,
    TestContext,
};
use arduinox86_client::ServerCpuType;
use iced_x86::{Mnemonic, OpKind};
use moo::types::{MooCpuType, MooException, MooIvtOrder};

pub struct BusOps {
    ops: Vec<BusOp>,
}

impl From<&[MyServerCycleState]> for BusOps {
    fn from(cycle_states: &[MyServerCycleState]) -> Self {
        let mut bus_ops = Vec::new();

        let mut latched_bus_op = None;
        for cycle_state in cycle_states {
            if let Ok(bus_op) = BusOp::try_from(cycle_state) {
                log::trace!("Collected bus op: {:X?}", bus_op);
                latched_bus_op = Some(bus_op);
            }
            else {
                if let Some(mut latched_bus_op_inner) = latched_bus_op {
                    latched_bus_op_inner.data = cycle_state.data_bus();
                    latched_bus_op_inner.idx = bus_ops.len();
                    bus_ops.push(BusOp::from(latched_bus_op_inner));
                    latched_bus_op = None; // Reset the latched bus operation.
                }
            }
        }

        BusOps { ops: bus_ops }
    }
}

impl BusOps {
    pub fn new(ops: &[BusOp]) -> Self {
        let ops = ops.to_vec();
        BusOps { ops }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn log(&self, context: &mut TestContext) {
        trace_log!(context, "Bus operations ({})", self.len());
        for (i, bus_op) in self.ops.iter().enumerate() {
            trace_log!(
                context,
                "{:02}: Addr: {:06X}, Data: {:04X?}, Type: {:?}",
                i,
                bus_op.addr,
                bus_op.data,
                bus_op.op_type
            );
        }
    }

    pub fn ops(&self) -> &[BusOp] {
        &self.ops
    }

    pub fn validate(
        &self,
        config: &Config,
        registers: &Registers,
        opcode: u8,
        instruction: &iced_x86::Instruction,
        op0: OpKind,
        op1: OpKind,
    ) -> anyhow::Result<()> {
        let has_memory_read = self.ops.iter().any(|op| op.op_type == BusOpType::MemRead);
        let has_memory_write = self.ops.iter().any(|op| op.op_type == BusOpType::MemWrite);

        match op0 {
            OpKind::Memory => {
                if !has_memory_read {
                    if matches!(config.test_gen.cpu_type, MooCpuType::Intel80286)
                        && config.test_gen.esc_opcodes.contains(&opcode)
                    {
                        // 80286 ESC instructions do not automatically read memory.
                    }
                    else if !matches!(instruction.mnemonic(), Mnemonic::Mov) {
                        // Mov just overwrites its operand, so we don't need a read.
                        return Err(anyhow::anyhow!(
                            "Expected memory read operation for Op0, but none found."
                        ));
                    }
                }

                if !has_memory_write {
                    if matches!(config.test_gen.cpu_type, MooCpuType::Intel80286)
                        && config.test_gen.esc_opcodes.contains(&opcode)
                    {
                        // Okay
                    }
                    else {
                        match instruction.mnemonic() {
                            Mnemonic::Jmp
                            | Mnemonic::Test
                            | Mnemonic::Cmp
                            | Mnemonic::Xlatb
                            | Mnemonic::Mul
                            | Mnemonic::Imul
                            | Mnemonic::Div
                            | Mnemonic::Idiv => {
                                // These mnemonics have a memory operand0 without a write operation.
                            }
                            Mnemonic::Rcl
                            | Mnemonic::Rcr
                            | Mnemonic::Shl
                            | Mnemonic::Shr
                            | Mnemonic::Sal
                            | Mnemonic::Sar
                            | Mnemonic::Rol
                            | Mnemonic::Ror => {
                                match op1 {
                                    OpKind::Immediate8 => {
                                        let masked_imm = (instruction.immediate8() as u16) & config.test_gen.shift_mask;

                                        if config.test_gen.writeless_null_shifts && (masked_imm == 0) {
                                            // Ok
                                        }
                                        else {
                                            return Err(anyhow::anyhow!(
                                                "Expected memory write operation for Op0, but none found. Masked imm8: {:04X}",
                                                masked_imm
                                            ));
                                        }
                                    }
                                    _ => {
                                        let masked_cx = registers.cx() & config.test_gen.shift_mask;
                                        // If masked cx is 0, these instructions won't write to memory.
                                        if config.test_gen.writeless_null_shifts && (masked_cx == 0) {
                                            // Ok
                                        }
                                        else {
                                            return Err(anyhow::anyhow!(
                                                "Expected memory write operation for Op0, but none found. Masked CX: {:04X}",
                                                masked_cx
                                            ));
                                        }
                                    }
                                }
                            }
                            _ => {
                                return Err(anyhow::anyhow!(
                                    "Expected memory write operation for Op0, but none found."
                                ));
                            }
                        }
                    }
                }
            }
            OpKind::NearBranch16 => {
                // Shouldn't have any reads or writes.
                if !matches!(instruction.mnemonic(), Mnemonic::Call) {
                    if has_memory_read || has_memory_write {
                        return Err(anyhow::anyhow!(
                            "Expected no memory operations for branch instruction {:?}, but found some.",
                            instruction.mnemonic()
                        ));
                    }
                }
            }
            _ => {}
        }

        if let OpKind::Memory = op1 {
            if !has_memory_read {
                if !matches!(instruction.mnemonic(), Mnemonic::Lea) {
                    return Err(anyhow::anyhow!(
                        "Expected memory read operation for Op1, but none found."
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn detect_exception(&self, context: &mut TestContext, cpu_type: ServerCpuType) -> Option<MooException> {
        // Check for an exception in the bus operations.

        let mut have_stack_frame = false;
        let mut flag_address = 0;
        let mut stack_frame_idx = 0;
        let mut ivt_read_idx = 0;

        let last_write = self
            .ops
            .iter()
            .rev()
            .find(|bus_op| matches!(bus_op.op_type, BusOpType::MemWrite));

        let last_consecutive_writes: Vec<_> = self
            .ops
            .iter()
            .rev()
            .skip_while(|bus_op| !matches!(bus_op.op_type, BusOpType::MemWrite))
            .take_while(|bus_op| matches!(bus_op.op_type, BusOpType::MemWrite))
            .cloned() // if you want owned BusOps, drop if &BusOp is fine
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        if last_consecutive_writes.len() > 2 {
            trace_log!(
                context,
                "Have {} consecutive last writes from {:08X?} to {:08X?}. Likely exception stack frame.",
                last_consecutive_writes.len(),
                last_consecutive_writes.first().map(|op| op.addr).unwrap_or(0),
                last_consecutive_writes.last().map(|op| op.addr).unwrap_or(0)
            );

            if last_consecutive_writes.first().unwrap().addr & 1 != 0 {
                trace_log!(context, "Stack pointer appears unaligned.")
            }
        }

        if let Some(last_write) = last_write {
            let sp_is_odd = last_write.addr & 1 != 0;

            have_stack_frame = if sp_is_odd {
                // Look for six consecutive writes to the stack frame.
                self.ops.windows(6).rev().any(|window| {
                    let all_writes = window.iter().all(|op| op.op_type == BusOpType::MemWrite);

                    if all_writes {
                        flag_address = window[0].addr;
                        stack_frame_idx = window[0].idx;
                    }

                    all_writes
                })
            }
            else {
                // Look for three consecutive writes to the stack frame.
                self.ops.windows(3).rev().any(|window| {
                    let all_writes = window.iter().all(|op| op.op_type == BusOpType::MemWrite);

                    if all_writes {
                        flag_address = window[0].addr;
                        stack_frame_idx = window[0].idx;
                    }

                    all_writes
                })
            }
        }

        let mut exception_num = 0;
        let have_two_consecutive_ivr_reads = self.ops.windows(2).rev().any(|window| {
            let have_exception = window[0].op_type == BusOpType::MemRead
                && window[1].op_type == BusOpType::MemRead
                && window[0].addr < 0x1024
                && window[0].addr % 4 == 0
                && window[1].addr < 0x1024;
            if have_exception {
                exception_num = window[0].addr / 4;
                ivt_read_idx = window[0].idx;
            }
            have_exception
        });

        let mut have_exception = false;
        if have_stack_frame && have_two_consecutive_ivr_reads {
            let ivt_order = MooIvtOrder::from(cpu_type);
            match ivt_order {
                MooIvtOrder::ReadFirst => {
                    if ivt_read_idx < stack_frame_idx {
                        have_exception = true;
                    }
                }
                MooIvtOrder::PushFirst => {
                    if stack_frame_idx < ivt_read_idx {
                        have_exception = true;
                    }
                }
            }

            println!(
                "Have stack frame at bus op idx {} and IVT reads at bus op idx {}, exception num: {}, cpu_type: {:?} ivt_order: {:?} passed: {}",
                stack_frame_idx, ivt_read_idx, exception_num, cpu_type, ivt_order, have_exception
            );

            if have_exception {
                return Some(MooException {
                    exception_num: exception_num as u8,
                    flag_address,
                });
            }
        }
        None
    }
}
