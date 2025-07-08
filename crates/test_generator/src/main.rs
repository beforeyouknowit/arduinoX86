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

mod cpu_common;
mod display;
mod flags;
mod gen_opcode;
mod gen_regs;
mod gen_tests;
mod modrm;

use anyhow::Context;
//use iced_x86::{Decoder, DecoderOptions};
use ard808x_client::{CpuClient, RandomizeOpts, ServerCpuType};
use clap::Parser;
use rand::prelude::*;
use rand::{Rng, SeedableRng};
use serde::Deserialize;
use std::io::Write;
use std::{fs, path::PathBuf};

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CpuType {
    Intel8088,
    Intel8086,
    NecV20,
    NecV30,
    Intel80188,
    Intel80186,
    Intel80286,
}

impl CpuType {
    pub fn bitness(&self) -> u32 {
        // If we ever add 386 or later, we'll need to match
        16
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CpuMode {
    Real,
    Unreal,
    Protected,
}

impl From<CpuType> for ServerCpuType {
    fn from(cpu_type: CpuType) -> Self {
        match cpu_type {
            CpuType::Intel8088 => ServerCpuType::Intel8088,
            CpuType::Intel8086 => ServerCpuType::Intel8086,
            CpuType::NecV20 => ServerCpuType::NecV20,
            CpuType::NecV30 => ServerCpuType::NecV30,
            CpuType::Intel80188 => ServerCpuType::Intel80188(false),
            CpuType::Intel80186 => ServerCpuType::Intel80186(false),
            CpuType::Intel80286 => ServerCpuType::Intel80286,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TerminationCondition {
    Queue,
    Halt,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    test_gen: TestGen,
    test_exec: TestExec,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TestExec {
    polling_sleep: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TestGen {
    cpu_type: CpuType,
    cpu_mode: CpuMode,
    seed: u64,
    termination_condition: TerminationCondition,
    output_dir: PathBuf,
    address_mask: u32,
    instruction_address_range: [u32; 2],
    opcode_range: Vec<u8>,
    excluded_opcodes: Vec<u8>,
    prefixes: Vec<u8>,
    segment_prefixes: Vec<u8>,
    test_count: usize,
    append_file: bool,

    segment_override_chance: f32,
    lock_chance: f32,

    reg_zero_chance: f32,
    reg_ff_chance: f32,

    mem_zero_chance: f32,
    mem_ff_chance: f32,

    disable_seg_overrides: Vec<u8>,
    disable_lock_prefix: Vec<u8>,

    randomize_mem_interval: usize,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Path to the TOML config file
    #[arg(long, value_name = "FILE")]
    config_file: PathBuf,

    #[arg(long)]
    com_port: Option<String>,
}

pub struct TestContext {
    seed: u64,
    rng: StdRng,
    client: CpuClient,
}

pub enum Registers {
    V1(ard808x_client::RemoteCpuRegistersV1),
    V2(ard808x_client::RemoteCpuRegistersV2),
}

impl Registers {
    pub fn randomize(&mut self, opts: &RandomizeOpts, rand: &mut rand::rngs::StdRng) {
        match self {
            Registers::V1(regs) => {
                //gen_regs::randomize_v1(&self.context, &self.config.test_gen, regs);
            }
            Registers::V2(regs) => regs.randomize(opts, rand),
        }
    }

    pub fn to_buffer<W: Write>(&self, buf: &mut W) {
        match self {
            Registers::V1(regs) => {
                //gen_regs::write_v1(&mut W, regs);
                unimplemented!("Writing V1 registers to buffer is not implemented yet");
            }
            Registers::V2(regs) => regs.to_buffer(buf),
        }
    }

    pub fn buf_len(&self) -> usize {
        match self {
            Registers::V1(regs) => 28,
            Registers::V2(regs) => 102,
        }
    }

    pub fn calculate_code_address(&self) -> u32 {
        match self {
            Registers::V1(regs) => regs.calculate_code_address(),
            Registers::V2(regs) => regs.calculate_code_address(),
        }
    }

    pub fn normalize_descriptors(&mut self) {
        match self {
            Registers::V1(regs) => {}
            Registers::V2(regs) => regs.normalize_descriptors(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Parse command‐line args
    let cli = Cli::parse();

    // Read the file into a string
    let text = fs::read_to_string(&cli.config_file)
        .with_context(|| format!("reading {}", cli.config_file.display()))?;

    // Parse as TOML
    let config: Config = toml::from_str(&text).context("parsing TOML into Config")?;

    // Initialize the random number generator
    let seed: u64 = config.test_gen.seed;
    let mut rng = StdRng::seed_from_u64(seed);

    // Create a cpu_client connection to cpu_server.
    let mut cpu_client = match CpuClient::init(cli.com_port.clone()) {
        Ok(ard_client) => {
            println!("Opened connection to Arduino_8088 server!");
            ard_client
        }
        Err(e) => {
            eprintln!("Error connecting to Arduino_8088 server: {e}");
            std::process::exit(1);
        }
    };

    cpu_client.set_random_seed(seed as u32)?;
    cpu_client.randomize_memory()?;

    let mut context = TestContext {
        seed,
        rng,
        client: cpu_client,
    };

    gen_tests::gen_tests(&mut context, &config)?;

    Ok(())
}
