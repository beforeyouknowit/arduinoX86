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

use crate::{global_trace_banner, global_trace_log, trace_banner, trace_log, ExceptionSeenEntry, TestContext};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct GenerationStats {
    pub(crate) total: usize,
    pub(crate) modrm_ct: usize,
    pub(crate) modrm_exception_ct: usize,
    pub(crate) modrm_no_exception_ct: usize,
    pub(crate) address_mode_ct: usize,
    pub(crate) register_mode_ct: usize,
    pub(crate) sib_ct: usize,
    pub(crate) sib_exception_ct: usize,
    pub(crate) sib_no_exception_ct: usize,
    pub(crate) mnemonic_set: HashMap<String, usize>,
    pub(crate) exceptions: HashMap<ExceptionSeenEntry, usize>,
}

impl Default for GenerationStats {
    fn default() -> Self {
        GenerationStats {
            total: 0,
            modrm_ct: 0,
            modrm_exception_ct: 0,
            modrm_no_exception_ct: 0,
            address_mode_ct: 0,
            register_mode_ct: 0,
            sib_ct: 0,
            sib_exception_ct: 0,
            sib_no_exception_ct: 0,
            mnemonic_set: HashMap::new(),
            exceptions: HashMap::new(),
        }
    }
}

impl GenerationStats {
    pub fn add_mnemonic(&mut self, mnemonic: &str) {
        *self.mnemonic_set.entry(mnemonic.to_string()).or_insert(0) += 1;
    }

    pub fn add_exception(&mut self, exception: u8, sib: bool) {
        let entry = ExceptionSeenEntry {
            exception_number: exception,
            sib,
        };
        self.exceptions.entry(entry).and_modify(|e| *e += 1).or_insert(1);
    }

    pub fn exception_ct(&self) -> usize {
        self.exceptions.len()
    }

    pub fn most_frequent_mnemonic(&self) -> Option<(&String, &usize)> {
        self.mnemonic_set.iter().max_by_key(|entry| entry.1)
    }

    pub fn log(&self, context: &mut TestContext) {
        if let Some((mnemonic, count)) = self.most_frequent_mnemonic() {
            let mnemonic_str = format!("Most frequent mnemonic: {} ({} times)", mnemonic, count);
            global_trace_log!(context, "{}", mnemonic_str);
            log::debug!("GenerationStats::log(): {}", mnemonic_str);
        }

        let addressing_total = self.modrm_ct + self.sib_ct;
        if addressing_total > 0 {
            global_trace_log!(
                context,
                "Addressing mode breakdown: ModRM: {} ({:.2}%), SIB: {} ({:.2}%)",
                self.modrm_ct,
                (self.modrm_ct as f64 / addressing_total as f64) * 100.0,
                self.sib_ct,
                (self.sib_ct as f64 / addressing_total as f64) * 100.0
            );
        }

        // Collect a list of all exceptions (stripping SIB flag)
        let mut all_exceptions: Vec<(u8, usize)> = self
            .exceptions
            .iter()
            .fold(HashMap::new(), |mut m, (k, v)| {
                *m.entry(k.exception_number).or_insert(0) += v;
                m
            })
            .into_iter()
            .collect();

        all_exceptions.sort_by_key(|f| f.0);

        if all_exceptions.is_empty() {
            global_trace_log!(context, "No exceptions seen.");
        }
        else {
            global_trace_log!(context, "Exceptions seen (all):");
            for exception in all_exceptions {
                global_trace_log!(
                    context,
                    "{}: {:5}/{:5} ({:.2}%)",
                    exception.0,
                    exception.1,
                    self.total,
                    (exception.1 as f64 / self.total as f64) * 100.0
                );
            }
        }

        // Separate exception stats by SIB.
        let mut sib_exceptions = self
            .exceptions
            .iter()
            .filter(|(entry, _)| entry.sib)
            .collect::<Vec<(&ExceptionSeenEntry, &usize)>>();

        sib_exceptions.sort_by_key(|f| f.0);

        if !sib_exceptions.is_empty() {
            global_trace_log!(context, "Exceptions seen (SIB):");
            for (exception, count) in sib_exceptions {
                global_trace_log!(
                    context,
                    "{}: {:5}/{:5} ({:.2}%)",
                    exception.exception_number,
                    count,
                    self.total,
                    (*count as f64 / self.total as f64) * 100.0
                );
            }

            if self.modrm_ct > 0 {
                global_trace_log!(
                    context,
                    "ModRM exception rate: {:.2}% ({}/{})",
                    (self.modrm_exception_ct as f64 / self.modrm_ct as f64) * 100.0,
                    self.modrm_exception_ct,
                    self.modrm_ct,
                );
            }

            if self.sib_ct > 0 {
                global_trace_log!(
                    context,
                    "SIB exception rate: {:.2}% ({}/{})",
                    (self.sib_exception_ct as f64 / self.sib_ct as f64) * 100.0,
                    self.sib_exception_ct,
                    self.sib_ct,
                );
            }
        }

        global_trace_banner!(context);
    }
}
