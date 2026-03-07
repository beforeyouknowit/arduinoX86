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
use anyhow::Context;
use moo::types::MooRamEntry;

pub fn write_initial_mem(ctx: &mut TestContext, initial_mem: &[MooRamEntry]) -> anyhow::Result<()> {
    let mut last_mem_address = 0;
    let mut mem_vec: Vec<(u32, Vec<u8>)> = Vec::new();
    let mut consecutive_start_address = 0;
    let mut consecutive_bytes = Vec::new();
    // Make concurrent vectors out of consecutive memory entries.
    for entry in initial_mem {
        if entry.address == last_mem_address + 1 {
            // Consecutive entry.
            consecutive_bytes.push(entry.value);
        }
        else {
            // Push the previous consecutive entries, if any.
            if !consecutive_bytes.is_empty() {
                mem_vec.push((consecutive_start_address, consecutive_bytes.clone()));
                consecutive_bytes.clear();
            }
            consecutive_start_address = entry.address;
            consecutive_bytes.push(entry.value);
        }
        last_mem_address = entry.address;
    }

    // Push the last consecutive entries, if any.
    if !consecutive_bytes.is_empty() {
        mem_vec.push((consecutive_start_address, consecutive_bytes));
    }

    for span in mem_vec {
        trace_log!(
            ctx,
            "Writing initial memory at address {:08X} with {} bytes: {:02X?}",
            span.0,
            span.1.len(),
            span.1
        );
        ctx.client
            .set_memory(span.0, &span.1)
            .with_context(|| format!("Writing initial memory at address {:08X}", span.0))?;
    }
    Ok(())
}
