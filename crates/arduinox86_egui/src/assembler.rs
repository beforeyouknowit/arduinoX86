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
use std::{
    env,
    error::Error,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tempfile::NamedTempFile;

/// A little wrapper around calling `nasm` from Rust.
pub struct Assembler {
    nasm_path: PathBuf,
    format: String,
    stdout_str: String,
    stderr_str: String,
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new("bin")
    }
}

impl Assembler {
    pub fn new<S: Into<String>>(format: S) -> Self {
        let nasm_path = if let Ok(p) = env::var("NASM_PATH") {
            PathBuf::from(p)
        }
        else {
            PathBuf::from("./nasm")
        };

        Assembler {
            nasm_path,
            format: format.into(),
            stdout_str: String::new(),
            stderr_str: String::new(),
        }
    }

    /// Override where to find the `nasm` executable.
    pub fn with_nasm_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.nasm_path = path.into();
        self
    }

    /// Assemble the given in-memory assembly (as `&str`) into `output_path`.
    ///
    /// # Errors
    ///
    /// Returns an error if spawning `nasm` fails or if NASM exits with a non-zero code.
    pub fn assemble_str<P: AsRef<Path>>(&mut self, asm: &str, output_path: P) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut temp_file = NamedTempFile::with_suffix(".asm")?;
        // Write the assembly code to a temporary file
        temp_file.write_all(asm.as_bytes())?;
        // Get the path to the temporary file

        let input_path = temp_file.path();

        log::debug!("Assembling from temporary file: {}", input_path.display());

        let mut child = Command::new(&self.nasm_path)
            .arg(input_path)
            .arg("-f")
            .arg(&self.format)
            .arg("-o")
            .arg(output_path.as_ref())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(asm.as_bytes())?;
            // drop(stdin) to close
        }

        let output = child.wait_with_output()?;

        self.stdout_str = String::from_utf8_lossy(&output.stdout).into_owned();
        self.stderr_str = String::from_utf8_lossy(&output.stderr).into_owned();

        if output.status.success() {
            log::debug!("assemble_str(): nasm call succeeded!");
        }
        else {
            let error_str = format!(
                "nasm failed with exit code {}: {}",
                output.status.code().unwrap_or(-1),
                self.stderr_str
            );
            log::error!("{}", error_str);
            self.stderr_str = error_str;

            return Err(self.stderr_str.clone().into());
        }

        let mut binary = Vec::new();
        if let Ok(mut file) = std::fs::File::open(output_path) {
            use std::io::Read;
            file.read_to_end(&mut binary)?;
        }
        else {
            return Err("Failed to read output binary file".into());
        }

        Ok(binary)
    }

    pub fn stdout(&self) -> &str {
        &self.stdout_str
    }

    pub fn stderr(&self) -> &str {
        &self.stderr_str
    }
}
