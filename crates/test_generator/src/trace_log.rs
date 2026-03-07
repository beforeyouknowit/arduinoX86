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
use crate::{trace_log, TestContext};
use moo::{prelude::MooCycleState, types::MooCycleStatePrinter};

// pub fn log_bus_ops(ctx: &mut TestContext, bus_ops: &[BusOp]) {
//     trace_log!(ctx, "Bus operations ({})", bus_ops.len());
//     for (i, bus_op) in bus_ops.iter().enumerate() {
//         trace_log!(
//             ctx,
//             "{:02}: Addr: {:06X}, Data: {:04X?}, Type: {:?}",
//             i,
//             bus_op.addr,
//             bus_op.data,
//             bus_op.op_type
//         );
//     }
// }

pub fn log_cycle_states(ctx: &mut TestContext, cycles: &[MooCycleState]) {
    let mut address_latch = 0;
    for (ci, cycle) in cycles.iter().enumerate() {
        if cycle.pins0 & MooCycleState::PIN_ALE != 0 {
            address_latch = cycle.address_bus;
        }

        trace_log!(
            ctx,
            "{}",
            MooCycleStatePrinter {
                cpu_type: ctx.server_cpu.into(),
                address_latch,
                state: cycle.clone(),
                show_cycle_num: true,
                cycle_num: ci,
            }
        );
    }
}
