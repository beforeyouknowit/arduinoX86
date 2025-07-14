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
mod cycles;
mod display;
mod flags;
mod gen_regs;
mod gen_tests;
mod instruction;
mod modrm;
mod registers;
mod state;

use anyhow::Context;
//use iced_x86::{Decoder, DecoderOptions};
use ard808x_client::{CpuClient, ProgramState, RegisterSetType, ServerCpuType};
use clap::Parser;
use moo::types::MooCpuType;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::time::Instant;
use std::{fs, path::PathBuf};

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum CpuMode {
    Real,
    Unreal,
    Protected,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum TerminationCondition {
    Queue,
    Halt,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpcodeMetadata {
    status: String,
    arch: String,
    flags: Option<String>,
    flags_mask: Option<u32>,
    reg: Option<HashMap<String, OpcodeMetadata>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TestMetadata {
    repo: String,
    version: String,
    syntax_version: u32,
    cpu: String,
    cpu_detail: String,
    generator: String,
    author: String,
    date: String,
    opcodes: HashMap<String, OpcodeMetadata>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CountOverride {
    count: usize,
    opcode_range: [u8; 2],
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupExtensionOverride {
    opcode: u8,
    group_extension_range: [u8; 2],
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    test_gen: TestGen,
    test_exec: TestExec,
    metadata: TestMetadata,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TestExec {
    polling_sleep: u32,
    max_gen: u32,
    test_retry: u32,
    load_retry: u32,
    print_instruction: bool,
    print_initial_regs: bool,
    print_final_regs: bool,
    show_gen_time: bool,
    serial_debug_default: bool,
    serial_debug_test: Option<usize>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TestGen {
    cpu_type: MooCpuType,
    cpu_mode: CpuMode,
    base_seed: u64,
    termination_condition: TerminationCondition,
    output_dir: PathBuf,
    trace_file: PathBuf,
    moo_version: u8,
    moo_arch: String,

    address_mask: u32,
    ip_mask: u16,
    instruction_address_range: [u32; 2],

    extended_opcode: bool,
    opcode_range: Vec<u8>,
    group_extension_range: [u8; 2],
    group_extension_overrides: Vec<GroupExtensionOverride>,

    excluded_opcodes: Vec<u8>,

    test_count: usize,
    append_file: bool,

    writeless_null_shifts: bool,
    shift_mask: u16,

    register_beta: [f64; 2],
    max_prefixes: usize,
    prefix_beta: [f64; 2],

    lock_prefix_chance: f32,
    lock_prefix_opcode: u8,
    rep_prefix_chance: f32,

    reg_zero_chance: f32,
    reg_ff_chance: f32,

    sp_odd_chance: f32,
    sp_min_value: u16,
    sp_max_value: u16,
    mem_zero_chance: f32,
    mem_ones_chance: f32,
    mem_strategy_start: u32,
    mem_strategy_end: u32,

    extended_prefix: u8,
    group_opcodes: Vec<u8>,
    extended_group_opcodes: Vec<u8>,
    esc_opcodes: Vec<u8>,
    flow_control_opcodes: Vec<u8>,
    prefixes: Vec<u8>,
    segment_prefixes: Vec<u8>,
    rep_prefixes: Vec<u8>,
    rep_opcodes: Vec<u8>,
    rep_cx_mask: u16,

    disable_seg_overrides: Vec<u8>,
    disable_lock_prefix: Vec<u8>,

    count_overrides: Vec<CountOverride>,

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

    #[arg(long)]
    dry_run: bool,
}

pub struct TestContext {
    client: CpuClient,
    load_register_buffer: Cursor<Vec<u8>>,
    store_register_buffer: Vec<u8>,
    server_cpu: ServerCpuType,
    register_set_type: RegisterSetType,

    gen_start: Instant,
    gen_stop: Instant,
    trace_log: BufWriter<File>,

    dry_run: bool,
    last_program_state: Option<ProgramState>,
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

    // Create a cpu_client connection to cpu_server.
    let mut cpu_client = match CpuClient::init(cli.com_port.clone(), Some(2000)) {
        Ok(ard_client) => {
            println!("Opened connection to Arduino_8088 server!");
            ard_client
        }
        Err(e) => {
            eprintln!("Error connecting to Arduino_8088 server: {e}");
            std::process::exit(1);
        }
    };

    let server_cpu = ServerCpuType::from(config.test_gen.cpu_type);

    // Create a BufWriter using the trace log file.
    let trace_log_path = config
        .test_gen
        .output_dir
        .join(config.test_gen.trace_file.clone());
    let trace_log_file = File::create(&trace_log_path)
        .with_context(|| format!("Creating trace log file: {}", trace_log_path.display()))?;
    let trace_log = BufWriter::new(trace_log_file);

    let mut context = TestContext {
        client: cpu_client,
        load_register_buffer: Cursor::new(vec![0; 102]),
        store_register_buffer: vec![0; 102],
        server_cpu,
        register_set_type: RegisterSetType::from(server_cpu),
        gen_start: Instant::now(),
        gen_stop: Instant::now(),
        trace_log,
        dry_run: cli.dry_run,
        last_program_state: None,
    };

    gen_tests::gen_tests(&mut context, &config)?;

    Ok(())
}
