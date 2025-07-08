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

use crate::{Registers, TestContext, TestGen};
use ard808x_client::RandomizeOpts;

pub fn randomize_v2(context: &mut TestContext, config: TestGen, regs: &mut Registers) {
    let random_opts = RandomizeOpts {
        weight_zero: config.reg_zero_chance,
        weight_ones: config.reg_ff_chance,
        randomize_flags: true,
        clear_trap_flag: true,
        clear_interrupt_flag: true,
        randomize_general: true,
        randomize_x: false,
        randomize_msw: false,
        randomize_tr: false,
        randomize_ldt: false,
        randomize_segment_descriptors: false,
        randomize_table_descriptors: false,
    };

    regs.randomize(&random_opts, &mut context.rng);
}
