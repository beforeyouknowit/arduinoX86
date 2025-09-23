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

use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};

/// Returns a filename like `2025-09-23T13-45-02Z_trace.log`.
pub fn timestamped_filename(prefix: &str, suffix: &str) -> String {
    let now = OffsetDateTime::now_local().expect("Getting local time failed");
    // ISO-like, but with '-' instead of ':' to be Windows-safe.
    // Example: 2025-09-23T13-45-02Z
    const FMT: &[FormatItem<'_>] = format_description!("[year]-[month]-[day]T[hour]-[minute]-[second]Z");
    let ts = now.format(&FMT).expect("formatting timestamp failed");
    format!("{prefix}{ts}{suffix}")
}
