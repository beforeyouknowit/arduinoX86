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

mod bus_ops;
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
use arduinox86_client::{CpuClient, ProgramState, RegisterSetType, ServerCpuType};
use clap::Parser;
use moo::types::MooCpuType;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    fs::File,
    io::{BufWriter, Cursor},
    path::PathBuf,
    time::Instant,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum InstructionSize {
    Sixteen,
    ThirtyTwo,
}

impl From<InstructionSize> for u32 {
    fn from(size: InstructionSize) -> Self {
        match size {
            InstructionSize::Sixteen => 16,
            InstructionSize::ThirtyTwo => 32,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum CpuMode {
    Real,
    Unreal,
    Protected,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
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
pub struct StackPointerOverride {
    opcode: u8,
    min:    u16,
    max:    u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModRmOverride {
    opcode: u8,
    mask: u8,
    invalid_chance: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    test_gen:  TestGen,
    test_exec: TestExec,
    metadata:  TestMetadata,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TestExec {
    polling_sleep: u32,
    validate_count: u32,
    max_gen: u32,
    test_retry: u32,
    load_retry: u32,
    test_timeout: u32,
    print_instruction: bool,
    print_initial_regs: bool,
    print_final_regs: bool,
    show_gen_time: bool,
    serial_timeout: u32,
    serial_debug_default: bool,
    serial_debug_test: Option<usize>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TestGen {
    set_version_major: u8,
    set_version_minor: u8,
    cpu_type: MooCpuType,
    cpu_mode: CpuMode,
    base_seed: u64,
    termination_condition: TerminationCondition,
    test_output_dir: PathBuf,
    trace_output_dir: PathBuf,
    verify_trace_output_dir: PathBuf,
    trace_file_suffix: PathBuf,
    moo_version: u8,
    moo_arch: String,
    gen_widths: Vec<InstructionSize>,

    address_mask: u32,
    ip_mask: u16,
    instruction_address_range: [u32; 2],

    extended_opcode: bool,
    opcode_range: Vec<u8>,
    group_extension_range: [u8; 2],
    group_extension_overrides: Vec<GroupExtensionOverride>,

    excluded_opcodes:    Vec<u8>,
    exclude_esc_opcodes: bool,

    test_count:  usize,
    append_file: bool,

    writeless_null_shifts: bool,
    shift_mask: u16,

    register_beta: [f64; 2],
    max_prefixes:  usize,
    prefix_beta:   [f64; 2],

    lock_prefix_chance: f32,
    lock_prefix_opcode: u8,
    rep_prefix_chance:  f32,

    reg_zero_chance:  f32,
    reg_ones_chance:  f32,
    imm_zero_chance:  f32,
    imm_ones_chance:  f32,
    imm8s_min_chance: f32,
    imm8s_max_chance: f32,
    near_branch_ban:  u16,

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
    operand_size_prefix: Option<u8>,
    address_size_prefix: Option<u8>,
    rep_prefixes: Vec<u8>,
    rep_opcodes: Vec<u8>,
    rep_cx_mask: u16,

    disable_seg_overrides: Vec<u8>,
    disable_lock_prefix:   Vec<u8>,

    sp_overrides:    Vec<StackPointerOverride>,
    modrm_overrides: Vec<ModRmOverride>,
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

    #[arg(long)]
    validate: bool,
}

pub struct TestContext {
    client: CpuClient,
    load_register_buffer: Cursor<Vec<u8>>,
    store_register_buffer: Vec<u8>,
    server_cpu: ServerCpuType,
    register_set_type: RegisterSetType,

    file_seed: u64,
    gen_start: Instant,
    gen_stop: Instant,
    gen_ct: usize,
    trace_log: BufWriter<File>,
    mnemonic_set: HashMap<String, usize>,

    dry_run: bool,
    last_program_state: Option<ProgramState>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Parse command‐line args
    let cli = Cli::parse();

    // Read the file into a string
    let text =
        fs::read_to_string(&cli.config_file).with_context(|| format!("reading {}", cli.config_file.display()))?;

    // Parse as TOML
    let mut config: Config = toml::from_str(&text).context("parsing TOML into Config")?;

    // Initialize the random number generator

    // Create a cpu_client connection to cpu_server.
    let mut cpu_client = match CpuClient::init(cli.com_port.clone(), Some(config.test_exec.serial_timeout as u64)) {
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

    // Create the trace output directory if it doesn't exist.
    if !config.test_gen.trace_output_dir.exists() {
        fs::create_dir_all(&config.test_gen.trace_output_dir).with_context(|| {
            format!(
                "Creating trace output directory: {}",
                config.test_gen.trace_output_dir.display()
            )
        })?;
    }
    if !config.test_gen.verify_trace_output_dir.exists() {
        fs::create_dir_all(&config.test_gen.verify_trace_output_dir).with_context(|| {
            format!(
                "Creating trace output directory: {}",
                config.test_gen.verify_trace_output_dir.display()
            )
        })?;
    }
    let trace_filename = PathBuf::from(format!("init{}", config.test_gen.trace_file_suffix.clone().display()));

    // Create a BufWriter using the trace log file.
    let trace_log_path = config.test_gen.trace_output_dir.join(trace_filename);
    let trace_log_file = File::create(&trace_log_path)
        .with_context(|| format!("Creating trace log file: {}", trace_log_path.display()))?;
    let trace_log = BufWriter::new(trace_log_file);

    let (load_register_buffer, store_register_buffer) = match config.test_gen.cpu_type {
        MooCpuType::Intel80286 => (Cursor::new(vec![0; 102]), vec![0; 102]),
        MooCpuType::Intel80386Ex => (Cursor::new(vec![0; 204]), vec![0; 208]),
        _ => {
            eprintln!("Unsupported CPU type: {:?}", config.test_gen.cpu_type);
            std::process::exit(1);
        }
    };

    let mut context = TestContext {
        client: cpu_client,
        load_register_buffer,
        store_register_buffer,
        server_cpu,
        register_set_type: RegisterSetType::from(server_cpu),
        file_seed: 0,
        gen_start: Instant::now(),
        gen_stop: Instant::now(),
        gen_ct: 0,
        trace_log,
        mnemonic_set: Default::default(),
        dry_run: cli.dry_run,
        last_program_state: None,
    };

    if config.test_gen.exclude_esc_opcodes {
        config
            .test_gen
            .excluded_opcodes
            .extend(config.test_gen.esc_opcodes.clone());
    }

    if cli.validate {
        gen_tests::validate_tests(&mut context, &config)?;
    }
    else {
        gen_tests::gen_tests(&mut context, &config)?;
    }

    Ok(())
}
