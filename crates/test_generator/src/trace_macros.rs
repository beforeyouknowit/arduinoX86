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

#[macro_export]
macro_rules! trace_banner {
    ($ctx:expr) => {{
        use std::io::Write;
        writeln!(
            $ctx.trace_log,
            ">>> ----------------------------------------------------------------------------------------------------"
        )
        .expect("failed to write to trace_log!");
    }};
}

#[macro_export]
macro_rules! global_trace_banner {
    ($ctx:expr) => {{
        use std::io::Write;
        writeln!(
            $ctx.global_trace_log,
            ">>> ----------------------------------------------------------------------------------------------------"
        )
        .expect("failed to write to global_trace_log!");
    }};
}

#[macro_export]
macro_rules! trace_log {
    // take a mutable Context (or &mut Context) and a format+args
    ($ctx:expr, $($arg:tt)*) => {{
        // bring Write into scope so write!/writeln! work
        use std::io::Write;
        // write the formatted text plus a newline
        writeln!($ctx.trace_log, $($arg)*)
            .expect("failed to write to trace_log!");
    }};
}

#[macro_export]
macro_rules! global_trace_log {
    // take a mutable Context (or &mut Context) and a format+args
    ($ctx:expr, $($arg:tt)*) => {{
        // bring Write into scope so write!/writeln! work
        use std::io::Write;
        // write the formatted text plus a newline
        writeln!($ctx.trace_log, $($arg)*)
            .expect("failed to write to trace_log!");
        writeln!($ctx.global_trace_log, $($arg)*)
            .expect("failed to write to trace_log!");
    }};
}

#[macro_export]
macro_rules! trace_flush {
    // take a mutable Context (or &mut Context) and a format+args
    ($ctx:expr) => {{
        // bring Write into scope so write!/writeln! work
        use std::io::Write;
        // write the formatted text plus a newline
        $ctx.trace_log.flush().expect("failed to flush trace_log!");
        $ctx.global_trace_log
            .flush()
            .expect("failed to flush global_trace_log!");
    }};
}

#[macro_export]
macro_rules! trace_error {
    ($ctx:expr, $($arg:tt)*) => {{
        use std::io::Write;
        // 1) prefix
        write!($ctx.trace_log, "## ERROR: ")
            .expect("failed to write error prefix to trace_log");
        // 2) the user’s format + newline
        writeln!($ctx.trace_log, $($arg)*)
            .expect("failed to write to trace_log");

        write!($ctx.global_trace_log, "## ERROR: ")
            .expect("failed to write error prefix to trace_log");
        // 2) the user’s format + newline
        writeln!($ctx.global_trace_log, $($arg)*)
            .expect("failed to write to trace_log");
        // 3) also log via log::error!
        log::error!($($arg)*);
    }};
}
