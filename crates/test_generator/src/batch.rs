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
    config::TestGen,
    enums::CpuMode,
    global_trace_banner,
    global_trace_log,
    size_prefix::TestOpcodeSizePrefix,
    TestContext,
};
use marty_dasm::Opcode;
use marty_isadb::IsaDB;
use std::{ffi::OsString, path::Path};

pub struct TestCandidateList {
    candidates: Vec<TestCandidate>,
}

impl TestCandidateList {
    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, TestCandidate> {
        self.candidates.iter()
    }

    /// Build a full list of tests that pass all the filtering criteria and batching parameters
    pub fn collect(ctx: &mut TestContext) -> Self {
        let cfg = &ctx.cfg.test_gen;

        // If user specifies a specific opcode override, expand only that one.
        if let Some(ov) = cfg.opcode_override {
            return TestCandidateList {
                candidates: expand_opcode_to_candidates(cfg, &ctx.isa_db, ov),
            };
        }

        let mut start = cfg.opcode_range[0];
        let mut end = cfg.opcode_range[1];
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }

        let mut out: Vec<TestCandidate> = Vec::with_capacity(4096);

        for opcode_raw in start..=end {
            // Skip explicitly excluded opcodes
            if cfg.excluded_opcodes.contains(&opcode_raw) {
                continue;
            }

            // Configuration must list this opcode as valid
            if !cfg.valid_opcodes.contains(&opcode_raw) {
                continue;
            }

            if cfg.skip_io && cfg.io_opcodes.contains(&opcode_raw) {
                continue;
            }

            // Skip any bare prefixes
            if opcode_raw < 0x100 {
                let b = opcode_raw as u8;
                if cfg.prefixes.contains(&b) {
                    continue;
                }
            }

            // Skip protected mode only opcodes if CPU mode is real/unreal
            match cfg.cpu_mode {
                CpuMode::Real | CpuMode::Unreal => {
                    if cfg.protected_mode_opcodes.contains(&opcode_raw) {
                        continue;
                    }
                }
                _ => {}
            }

            // Expand this opcode into (size_prefix[, group_ext]) units
            out.extend(expand_opcode_to_candidates(cfg, &ctx.isa_db, opcode_raw));
        }

        TestCandidateList {
            candidates: create_batch(&out, ctx.job_ct, ctx.job_no),
        }
    }

    pub fn log(&self, ctx: &mut TestContext) {
        let test_ct = self.test_ct(ctx);
        let cfg = &ctx.cfg.test_gen;
        global_trace_log!(
            ctx,
            "Generated {} test candidate files for CPU {:?}, mode {:?}, test count: {}, opcode range {:#06X}-{:#06X}, batch {}/{}",
            self.len(),
            cfg.cpu_type,
            cfg.cpu_mode,
            test_ct,
            cfg.opcode_range[0],
            cfg.opcode_range[1],
            ctx.job_no + 1,
            ctx.job_ct
        );

        for (i, c) in self.candidates.iter().enumerate() {
            global_trace_log!(ctx, "    {:04}: {}", i, c.filename().to_string_lossy());
        }

        global_trace_banner!(ctx);
    }

    pub fn test_ct(&self, ctx: &mut TestContext) -> usize {
        let mut total = 0;
        for c in self.candidates.iter() {
            let ct = ctx.cfg.test_gen.get_test_count(c.opcode);
            total += ct;
        }
        total
    }

    pub fn filter_validated(&mut self, ctx: &TestContext) {
        let path = ctx.validate_output_path.clone();
        self.filter_existing(path);
    }

    /// Filter the current candidate list to remove any whose output file already exists in the
    /// specified path.
    pub fn filter_existing(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        self.candidates.retain(|tc| {
            let file_path = path.join(tc.filename());
            !file_path.exists()
        });
    }
}

#[derive(Clone, Debug)]
pub struct TestCandidate {
    pub opcode: Opcode,
    pub size_prefix: TestOpcodeSizePrefix,
    pub opcode_extension: Option<u8>, // Some(0..=7) for group opcodes, None otherwise
}

impl TestCandidate {
    pub fn filename(&self) -> OsString {
        OsString::from(format!(
            "{}{}{}.MOO",
            self.size_prefix.to_filename_prefix(),
            self.opcode,
            self.op_ext_str()
        ))
    }

    pub fn trace_filename(&self, suffix: impl AsRef<Path>) -> OsString {
        OsString::from(format!(
            "{}{}{}{}",
            self.size_prefix.to_filename_prefix(),
            self.opcode,
            self.op_ext_str(),
            suffix.as_ref().display()
        ))
    }

    fn op_ext_str(&self) -> String {
        if let Some(ext) = self.opcode_extension {
            format!(".{:1}", ext)
        }
        else {
            "".to_string()
        }
    }
}

/// Expand a single opcode into all combinations of size prefix and extension
fn expand_opcode_to_candidates(cfg: &TestGen, isa_db: &IsaDB, opcode_raw: u16) -> Vec<TestCandidate> {
    let opcode = Opcode::from(opcode_raw);
    let mut v = Vec::with_capacity(32); // 4 * 8 = 32 max combinations

    for size_prefix in TestOpcodeSizePrefix::iter(
        cfg.cpu_type,
        opcode,
        &isa_db,
        &cfg.disable_operand_size_prefix,
        &cfg.disable_address_size_prefix,
    ) {
        if cfg.group_opcodes.contains(&opcode_raw) {
            let (op_ext_start, op_ext_end) = cfg.get_group_extension_range(opcode.into());
            for ext in op_ext_start..=op_ext_end {
                v.push(TestCandidate {
                    opcode: Opcode::from(opcode_raw),
                    size_prefix,
                    opcode_extension: Some(ext as u8),
                });
            }
        }
        else {
            v.push(TestCandidate {
                opcode: Opcode::from(opcode_raw),
                size_prefix,
                opcode_extension: None,
            });
        }
    }

    v
}

/// Create a batch of items from a larger list, based on the batch index and total number of batches.
/// If total is 1 or less, returns the full list.
/// Panics if idx >= total.
fn create_batch<T: Clone>(items: &[T], total: usize, idx: usize) -> Vec<T> {
    if total <= 1 {
        return items.to_vec();
    }
    assert!(idx < total, "board_number must be < total_boards");

    let n = items.len();
    if n == 0 {
        return Vec::new();
    }

    // Base chunk size and remainder to distribute
    let base = n / total;
    let rem = n % total;

    // Size of this batch: +1 if idx gets a remainder item
    let my_len = base + usize::from(idx < rem);

    // Start index: number of full base chunks before us, plus how many remainder items were assigned before us
    let start = idx * base + idx.min(rem);
    let end = start + my_len;

    items[start..end].to_vec()
}

#[cfg(test)]
mod tests {
    use super::create_batch;
    #[test]
    /// Test general case
    fn batches_cover_all_items_without_overlap() {
        let v: Vec<_> = (0..10).collect();
        let total = 3;

        let s0 = create_batch(&v, total, 0); // 4 items
        let s1 = create_batch(&v, total, 1); // 3 items
        let s2 = create_batch(&v, total, 2); // 3 items

        assert_eq!(s0, vec![0, 1, 2, 3]);
        assert_eq!(s1, vec![4, 5, 6]);
        assert_eq!(s2, vec![7, 8, 9]);

        let v: Vec<_> = (0..11).collect();
        let total = 3;

        let s0 = create_batch(&v, total, 0); // 4 items
        let s1 = create_batch(&v, total, 1); // 3 items
        let s2 = create_batch(&v, total, 2); // 3 items

        assert_eq!(s0, vec![0, 1, 2, 3]);
        assert_eq!(s1, vec![4, 5, 6, 7]);
        assert_eq!(s2, vec![8, 9, 10]);
    }

    #[test]
    /// Test that having more batches than items works correctly - extra batches are empty
    fn more_batches_than_items_is_ok() {
        let v: Vec<_> = (0..3).collect();
        let total = 5;

        let got: Vec<Vec<_>> = (0..total).map(|i| create_batch(&v, total, i)).collect();
        assert_eq!(got, vec![vec![0], vec![1], vec![2], vec![], vec![]]);
    }

    #[test]
    /// Test that we can provide an empty input list and get an empty list back
    fn empty_input() {
        let v: Vec<i32> = vec![];
        assert!(create_batch(&v, 4, 0).is_empty());
    }

    #[test]
    /// Test that providing total=0 returns the full list
    fn zero_total() {
        let v: Vec<_> = (0..10).collect();
        assert_eq!(create_batch(&v, 0, 0), v);
    }
}
