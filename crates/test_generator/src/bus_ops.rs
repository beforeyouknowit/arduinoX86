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
    cpu_common::{BusOp, BusOpPayload, BusOpType},
    cycles::MyServerCycleState,
    enums::{InstructionSize, TerminationCondition},
    hash_memory::HashMemory,
    registers::Registers,
    trace_error,
    trace_log,
    TestContext,
};
use anyhow::bail;
use arduinox86_client::ServerCpuType;
use iced_x86::{Mnemonic, OpKind};
use marty_dasm::Opcode;
use moo::types::{MooCpuType, MooException, MooIvtOrder};
use std::collections::HashMap;

pub struct BusOps {
    ops: Vec<BusOp>,
}

impl From<&[MyServerCycleState]> for BusOps {
    fn from(cycle_states: &[MyServerCycleState]) -> Self {
        let mut bus_ops = Vec::new();

        let mut latched_bus_op = None;
        for cycle_state in cycle_states {
            if let Ok(bus_op) = BusOp::try_from(cycle_state) {
                //log::trace!("Collected bus op: {:X?}", bus_op);
                latched_bus_op = Some(bus_op);
            }
            else {
                if let Some(mut latched_bus_op_inner) = latched_bus_op {
                    latched_bus_op_inner.data = cycle_state.data_bus();
                    latched_bus_op_inner.idx = bus_ops.len();
                    bus_ops.push(latched_bus_op_inner);
                    latched_bus_op = None; // Reset the latched bus operation.
                }
            }
        }

        if let Some(mut latched_bus_op_inner) = latched_bus_op {
            latched_bus_op_inner.idx = bus_ops.len();
            bus_ops.push(latched_bus_op_inner);
        }

        BusOps::new(&bus_ops)
    }
}

impl BusOps {
    pub const MAX_STACK_SIZE: usize = 16; // Maximum stack size in words.

    pub fn new(ops: &[BusOp]) -> Self {
        let ops = ops.to_vec();
        let mut bus_ops = BusOps { ops };

        bus_ops.detect_pairs();
        bus_ops
    }

    pub fn detect_pairs(&mut self) {
        let mut hash_table: HashMap<u32, Vec<usize>> = HashMap::new();

        for (i, op) in self.ops.iter().enumerate() {
            if matches!(op.op_type, BusOpType::MemRead | BusOpType::MemWrite) {
                hash_table.entry(op.addr).or_default().push(i);
            }
        }

        for entry in hash_table.values_mut() {
            if entry.len() >= 2 {
                for index in entry {
                    self.ops[*index].flags |= BusOp::FLAG_PAIRED;
                }
            }
        }
    }

    pub fn detect_pushes(&mut self, initial_regs: &Registers) {
        let stack_address = initial_regs.stack_address();
        let sp_aligned = stack_address & 0x01 == 0;
        let mut have_stack = false;
        let mut next_push = stack_address.wrapping_sub(2);

        for op in &mut self.ops {
            if matches!(op.op_type, BusOpType::MemWrite) {
                if op.addr < stack_address {
                    if sp_aligned {
                        if op.addr == next_push {
                            have_stack = true;
                            //log::warn!("Have probable stack push at address {:06X}", op.addr);
                            op.flags |= BusOp::FLAG_STACK;
                            next_push = next_push.wrapping_sub(2);
                        }
                        else if have_stack {
                            break;
                        }
                    }
                    else {
                        let sp_diff = stack_address.wrapping_sub(op.addr);

                        if sp_diff < Self::MAX_STACK_SIZE as u32 * 2 {
                            have_stack = true;
                            //log::warn!("Have probable (unaligned) stack push at address {:06X}", op.addr);
                            op.flags |= BusOp::FLAG_STACK;
                        }
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn log(&self, context: &mut TestContext) {
        trace_log!(context, "Bus operations ({})", self.len());
        for (i, bus_op) in self.ops.iter().enumerate() {
            let mut flag_string = String::new();

            if bus_op.flags & BusOp::FLAG_PAIRED != 0 {
                flag_string.push_str("PAIRED, ");
            }
            if bus_op.flags & BusOp::FLAG_STACK != 0 {
                flag_string.push_str("STACK, ");
            }

            trace_log!(
                context,
                "{:02}: Addr: {:06X}, Data: {:04X?}, Type: {:?} Flags [{:02X}]: {}",
                i,
                bus_op.addr,
                bus_op.data,
                bus_op.op_type,
                bus_op.flags,
                flag_string
            );
        }
    }

    pub fn ops(&self) -> &[BusOp] {
        &self.ops
    }

    pub fn validate(
        &self,
        context: &TestContext,
        registers: &Registers,
        opcode: Opcode,
        instruction: &iced_x86::Instruction,
        op0: OpKind,
        op1: OpKind,
        had_exception: bool,
    ) -> anyhow::Result<()> {
        let has_memory_read = self.ops.iter().any(|op| op.op_type == BusOpType::MemRead);
        let has_memory_write = self.ops.iter().any(|op| op.op_type == BusOpType::MemWrite);

        let config = &context.cfg;

        // The first bus operation should be a code fetch, unless prefetching is enabled.
        if !context.prefetch {
            if let Some(first_op) = self.ops.first() {
                if !(first_op.op_type == BusOpType::CodeRead && first_op.addr == registers.calculate_code_address()) {
                    return Err(anyhow::anyhow!(
                        "Expected first bus operation to be code fetch at address {:06X}, but found {:?} at {:06X}",
                        registers.eip(),
                        first_op.op_type,
                        first_op.addr
                    ));
                }
            }
        }

        // The last bus operation should be a HALT if halt termination was specified.
        if matches!(config.test_gen.termination_condition, TerminationCondition::Halt) {
            if let Some(last_op) = self.ops.last() {
                if last_op.op_type != BusOpType::Halt {
                    return Err(anyhow::anyhow!(
                        "Expected last bus operation to be HALT, but found {:?} at {:06X}",
                        last_op.op_type,
                        last_op.addr
                    ));
                }
            }
        }

        // Enforce a maximum cycle count if not a string operation.
        let is_string = matches!(
            instruction.mnemonic(),
            Mnemonic::Movsb
                | Mnemonic::Movsw
                | Mnemonic::Movsd
                | Mnemonic::Cmpsb
                | Mnemonic::Cmpsw
                | Mnemonic::Cmpsd
                | Mnemonic::Scasb
                | Mnemonic::Scasw
                | Mnemonic::Scasd
                | Mnemonic::Lodsb
                | Mnemonic::Lodsw
                | Mnemonic::Lodsd
                | Mnemonic::Stosb
                | Mnemonic::Stosw
                | Mnemonic::Stosd
        );

        let is_io_string = matches!(
            instruction.mnemonic(),
            Mnemonic::Insb | Mnemonic::Insw | Mnemonic::Insd | Mnemonic::Outsb | Mnemonic::Outsw | Mnemonic::Outsd
        );
        let is_enter = matches!(instruction.mnemonic(), Mnemonic::Enter);
        let is_pusha = matches!(instruction.mnemonic(), Mnemonic::Pusha | Mnemonic::Pushad);
        let is_popa = matches!(instruction.mnemonic(), Mnemonic::Popa | Mnemonic::Popad);
        let is_io_read = matches!(
            instruction.mnemonic(),
            Mnemonic::In | Mnemonic::Insb | Mnemonic::Insd | Mnemonic::Insw
        );
        let is_io_write = matches!(
            instruction.mnemonic(),
            Mnemonic::Out | Mnemonic::Outsb | Mnemonic::Outsw | Mnemonic::Outsd
        );
        let is_io = is_io_read || is_io_write;

        if !is_string && !is_io_string && !is_enter && !is_pusha && !is_popa {
            let non_code_ops = self
                .ops
                .iter()
                .filter(|op| op.op_type != BusOpType::CodeRead && op.op_type != BusOpType::Halt)
                .count();

            let max_bus_ops = config.test_gen.max_normal_bus_ops as usize;
            if non_code_ops > max_bus_ops {
                return Err(anyhow::anyhow!(
                    "Exceeded maximum bus operation count of {} for non-string, enter, or pusha instruction {:?}: actual bus ops = {}",
                    max_bus_ops,
                    instruction.mnemonic(),
                    non_code_ops
                ));
            }
        }

        match op0 {
            OpKind::Memory => {
                if !has_memory_read {
                    if matches!(config.test_gen.cpu_type, MooCpuType::Intel80286)
                        && config.test_gen.esc_opcodes.contains(&opcode.into())
                    {
                        // 80286 ESC instructions do not automatically read memory.
                    }
                    else if !matches!(
                        instruction.mnemonic(),
                        Mnemonic::Mov
                            | Mnemonic::Seto
                            | Mnemonic::Setno
                            | Mnemonic::Setb
                            | Mnemonic::Setae
                            | Mnemonic::Sete
                            | Mnemonic::Setne
                            | Mnemonic::Setbe
                            | Mnemonic::Seta
                            | Mnemonic::Sets
                            | Mnemonic::Setns
                            | Mnemonic::Setp
                            | Mnemonic::Setnp
                            | Mnemonic::Setl
                            | Mnemonic::Setge
                            | Mnemonic::Setle
                            | Mnemonic::Setg
                    ) {
                        // Mov just overwrites its operand, so we don't need a read.
                        return Err(anyhow::anyhow!(
                            "Expected memory read operation for Op0, but none found."
                        ));
                    }
                }

                if !has_memory_write {
                    if matches!(config.test_gen.cpu_type, MooCpuType::Intel80286)
                        && config.test_gen.esc_opcodes.contains(&opcode.into())
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
                            | Mnemonic::Idiv
                            | Mnemonic::Bt
                            | Mnemonic::Bts
                            | Mnemonic::Btr
                            | Mnemonic::Btc => {
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
            OpKind::NearBranch16 | OpKind::NearBranch32 => {
                // Shouldn't have any reads or writes.
                if !matches!(instruction.mnemonic(), Mnemonic::Call) {
                    if !had_exception && (has_memory_read || has_memory_write) {
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

        // Validate IO operations.
        for op in &self.ops {
            match op.op_type {
                BusOpType::MemWrite => {
                    // Check if this is an INS instruction writing to memory.
                    if is_io_string {
                        if op.flags & BusOp::FLAG_STACK == 0 {
                            // Bus operation is not a stack write.
                            match op.payload() {
                                Ok(BusOpPayload::Byte(byte)) => {
                                    if byte != 0xFF {
                                        return Err(anyhow::anyhow!(
                                            "Expected IO byte written to memory to be 0xFF, but found {:02X}",
                                            byte
                                        ));
                                    }
                                }
                                Ok(BusOpPayload::Word(word)) => {
                                    if word != 0xFFFF {
                                        return Err(anyhow::anyhow!(
                                            "Expected IO word written to memory to be 0xFFFF, but found {:04X}",
                                            word
                                        ));
                                    }
                                }
                                Err(e) => {
                                    return Err(anyhow::anyhow!("Failed to get IO MEMW payload: {}", e));
                                }
                            }
                        }
                    }
                }
                BusOpType::IoRead => {
                    if !is_io {
                        return Err(anyhow::anyhow!(
                            "Unexpected IO operation for non-IO instruction {:?}.",
                            instruction.mnemonic()
                        ));
                    }

                    if is_io_write {
                        return Err(anyhow::anyhow!(
                            "Unexpected IO read operation for IO write instruction {:?}.",
                            instruction.mnemonic()
                        ));
                    }

                    match op.payload() {
                        Ok(BusOpPayload::Byte(byte)) => {
                            if byte != 0xFF {
                                return Err(anyhow::anyhow!("Expected IO byte to be 0xFF, but found {:02X}", byte));
                            }
                        }
                        Ok(BusOpPayload::Word(word)) => {
                            if word != 0xFFFF {
                                return Err(anyhow::anyhow!("Expected IO word to be 0xFFFF, but found {:04X}", word));
                            }
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!("Failed to get IO busop payload: {}", e));
                        }
                    }
                }
                BusOpType::IoWrite => {
                    if !is_io {
                        return Err(anyhow::anyhow!(
                            "Unexpected IO operation for non-IO instruction {:?}.",
                            instruction.mnemonic()
                        ));
                    }

                    if is_io_read {
                        return Err(anyhow::anyhow!(
                            "Unexpected IO write operation for IO read instruction {:?}.",
                            instruction.mnemonic()
                        ));
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn hash_memory(&self) -> HashMemory {
        let mut memory = HashMemory::new();
        for op in &self.ops {
            memory.write_bus16(op.addr, op.bhe, op.data);
        }
        memory
    }

    pub fn detect_exception(
        &mut self,
        context: &mut TestContext,
        cpu_type: ServerCpuType,
        _operand_size: InstructionSize,
        init_regs: &Registers,
        final_regs: &Registers,
    ) -> anyhow::Result<Option<MooException>> {
        // Check for an exception in the bus operations.
        let hash_memory = self.hash_memory();
        let sp_address = init_regs.stack_address();
        let mut have_stack_frame = false;
        let flag_address;
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

        let last_writes_sorted = {
            let mut writes = last_consecutive_writes.clone();
            writes.sort_by_key(|op| op.addr);
            writes
        };

        let sp_unaligned = if init_regs.stack_address() & 0x01 != 0 {
            trace_log!(
                context,
                "Initial stack pointer is unaligned: {:08X}",
                init_regs.stack_address()
            );
            true
        }
        else {
            false
        };

        if final_regs.stack_address() < init_regs.stack_address() {
            trace_log!(
                context,
                "Stack grew down {} bytes, from {:08X} to {:08X}",
                init_regs.stack_address().wrapping_sub(final_regs.stack_address()),
                init_regs.stack_address(),
                final_regs.stack_address()
            )
        }
        else if init_regs.stack_address() < final_regs.stack_address() {
            trace_log!(
                context,
                "Stack shrank up {} bytes, from {:08X} to {:08X}",
                final_regs.stack_address().wrapping_sub(init_regs.stack_address()),
                init_regs.stack_address(),
                final_regs.stack_address()
            )
        }

        if last_consecutive_writes.len() > 2 {
            trace_log!(
                context,
                "Have {} consecutive last writes from {:08X?} to {:08X?}. Possible exception stack frame.",
                last_consecutive_writes.len(),
                last_writes_sorted.first().map(|op| op.addr).unwrap_or(0),
                last_writes_sorted.last().map(|op| op.addr).unwrap_or(0)
            );

            if last_writes_sorted.first().unwrap().flags & BusOp::FLAG_STACK != 0 {
                trace_log!(context, "Last writes include stack writes.");

                if last_writes_sorted.first().unwrap().addr & 1 != 0 {
                    if sp_unaligned {
                        trace_log!(context, "First stack write agrees that SP is unaligned.");
                    }
                    else {
                        trace_error!(context, "First stack write disagrees that SP is unaligned!");
                    }
                }
            }
        }

        if let Some(last_write) = last_write {
            let sp_is_odd = last_write.addr & 1 != 0;

            have_stack_frame = if sp_is_odd {
                // Look for six consecutive writes to the stack frame.
                self.ops.windows(6).rev().any(|window| {
                    let all_writes = window.iter().all(|op| op.op_type == BusOpType::MemWrite);

                    if all_writes {
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
                && window[0].addr < 0x0400
                && window[0].addr % 4 == 0
                && window[1].addr < 0x0400
                && window[1].addr > window[0].addr;
            if have_exception {
                exception_num = window[0].addr / 4;
                ivt_read_idx = window[0].idx;
            }
            have_exception
        });

        let mut have_exception = false;
        if have_stack_frame && have_two_consecutive_ivr_reads {
            // Look for 16-bit flag push.
            let first_push_addr = sp_address.saturating_sub(2);
            let flag_word = final_regs.flags();
            let found_word = hash_memory.read_u16(first_push_addr);

            if flag_word != found_word {
                let err_str = format!(
                    "Expected flags word {:04X} at first push address {:06X}, but found {:04X}",
                    flag_word, first_push_addr, found_word
                );
                trace_error!(context, "{}", err_str);
                bail!(err_str);
            }
            else {
                trace_log!(
                    context,
                    "Found expected flags word {:04X} at first push address {:06X}",
                    flag_word,
                    first_push_addr
                );
                flag_address = first_push_addr;
            }

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
                return Ok(Some(MooException {
                    exception_num: exception_num as u8,
                    flag_address,
                }));
            }
        }
        Ok(None)
    }
}
