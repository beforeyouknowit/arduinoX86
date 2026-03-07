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

mod batch;
mod bus_ops;
mod config;
mod cpu_common;
mod cycles;
mod disassembly;
mod display;
mod enums;
mod file;
mod flags;
mod generate;
mod hash_memory;
mod instruction;
mod modrm;
mod registers;
mod size_prefix;
mod state;
mod trace_log;
mod trace_macros;
mod validate;

use std::{
    fs,
    fs::File,
    io::{BufWriter, Cursor},
    path::PathBuf,
    time::Instant,
};

use arduinox86_client::{registers_common::SegmentSize, CpuClient, ProgramState, RegisterSetType, ServerCpuType};
use moo::types::MooCpuType;

use crate::{
    config::Config,
    enums::{CpuMode, ExecMode},
    file::timestamped_filename,
    generate::generation_stats::GenerationStats,
    size_prefix::TestOpcodeSizePrefix,
};
use anyhow::Context;
use clap::Parser;
use iced_x86::DecoderOptions;
use marty_isadb::IsaDB;
use rand::distr::weighted::WeightedIndex;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Path to the TOML config file
    #[arg(long, value_name = "FILE")]
    config_file: PathBuf,

    #[arg(long, default_value = "1")]
    num_boards: usize,

    #[arg(long, default_value = "0")]
    board_number: usize,

    #[arg(long)]
    com_port: Option<String>,

    #[arg(long)]
    dry_run: bool,

    #[arg(long)]
    validate: bool,

    #[arg(long)]
    only_modrm_overrides: bool,
}

#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct ExceptionSeenEntry {
    exception_number: u8,
    sib: bool,
}

pub struct TestContext {
    exec_mode: ExecMode,
    client: CpuClient,
    cfg: Config,
    job_ct: usize,
    job_no: usize,
    isa_db: IsaDB,
    load_register_buffer: Cursor<Vec<u8>>,
    store_register_buffer: Vec<u8>,
    server_cpu: ServerCpuType,
    register_set_type: RegisterSetType,
    test_opcode_size_prefix: TestOpcodeSizePrefix,
    code_segment_size: SegmentSize,
    file_seed: u64,
    prefetch: bool,
    gen_start: Instant,
    gen_stop: Instant,
    gen_ct: usize,
    gen_total: usize,
    file_gen_ct: usize,
    output_path: PathBuf,
    trace_path: PathBuf,
    validate_output_path: PathBuf,
    trace_log: BufWriter<File>,
    global_trace_log: BufWriter<File>,
    global_error_log: BufWriter<File>,
    iced_decoder_opts: u32,
    dry_run: bool,
    last_program_state: Option<ProgramState>,
    weighted_index: WeightedIndex<f32>,
    inject_values: Vec<u32>,
    stats: GenerationStats,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Parse command‐line args
    let cli = Cli::parse();

    if cli.board_number >= cli.num_boards {
        eprintln!("Error: board_number must be less than num_boards (0-offset)");
        std::process::exit(1);
    }

    // Extract config dir from config file path
    let config_dir = cli
        .config_file
        .parent()
        .context("getting config file parent directory")?;

    log::debug!("Using config dir: {}", config_dir.display());

    // Read the file into a string
    let text =
        fs::read_to_string(&cli.config_file).with_context(|| format!("reading {}", cli.config_file.display()))?;

    // Parse as TOML
    let mut config: Config = toml::from_str(&text).context("parsing TOML into Config")?;

    // Load the ISA database if specified
    let isa_db_path = if config.test_gen.isa_db.is_relative() {
        config_dir.join(&config.test_gen.isa_db)
    }
    else {
        config.test_gen.isa_db.clone()
    };

    if !isa_db_path.exists() {
        eprintln!("Error: ISA DB file not found: {}", isa_db_path.display());
        std::process::exit(1);
    }

    let marty_dasm_cpu = match config.test_gen.cpu_type {
        MooCpuType::Intel8088 | MooCpuType::Intel8086 => marty_dasm::CpuType::Intel808x,
        MooCpuType::Intel80188 | MooCpuType::Intel80186 => marty_dasm::CpuType::Intel8018x,
        MooCpuType::Intel80286 => marty_dasm::CpuType::Intel80286,
        MooCpuType::Intel80386Ex => marty_dasm::CpuType::Intel80386,
        _ => {
            eprintln!("Unsupported CPU type: {:?}", config.test_gen.cpu_type);
            std::process::exit(1);
        }
    };
    let isa_db = IsaDB::from_file(marty_dasm_cpu, &isa_db_path)
        .with_context(|| format!("loading ISA DB from {}", isa_db_path.display()))?;

    // Initialize the random number generator

    // Create a cpu_client connection to cpu_server.
    let cpu_client = match CpuClient::init(cli.com_port.clone(), Some(config.test_exec.serial_timeout as u64)) {
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
    let mode_suffix = config.test_gen.cpu_mode.to_path_suffix().to_string();

    // Create the test output directory if it doesn't exist.
    let output_dir_path = config.test_gen.test_output_dir.join(mode_suffix.clone());
    if !output_dir_path.exists() {
        log::warn!("Creating test output directory: {}", output_dir_path.display());
        fs::create_dir_all(&output_dir_path)
            .with_context(|| format!("Creating test output directory: {}", output_dir_path.display()))?;
    }

    // Create the trace output directory if it doesn't exist.
    let trace_dir_path = if cli.validate {
        config
            .test_gen
            .test_output_dir
            .join(mode_suffix.clone())
            .join(config.test_gen.validate_output_dir.clone())
            .join(config.test_gen.trace_output_dir.clone())
    }
    else {
        config
            .test_gen
            .test_output_dir
            .join(mode_suffix.clone())
            .join(config.test_gen.trace_output_dir.clone())
    };

    if !trace_dir_path.exists() {
        fs::create_dir_all(&trace_dir_path)
            .with_context(|| format!("Creating trace output directory: {}", trace_dir_path.display()))?;
    }

    let validate_output_dir = config
        .test_gen
        .test_output_dir
        .join(mode_suffix.clone())
        .join(config.test_gen.validate_output_dir.clone());
    if !validate_output_dir.exists() {
        fs::create_dir_all(&validate_output_dir).with_context(|| {
            format!(
                "Creating validation output directory: {}",
                validate_output_dir.display()
            )
        })?;
    }

    let trace_filename = PathBuf::from(format!("init{}", config.test_gen.trace_file_suffix.clone().display()));

    // Create a BufWriter using the trace log file.
    let trace_log_path = trace_dir_path.join(trace_filename);
    let trace_log_file = File::create(&trace_log_path)
        .with_context(|| format!("Creating trace log file: {}", trace_log_path.display()))?;
    let trace_log = BufWriter::new(trace_log_file);

    let global_trace_filename_prefix = format!("{}_", cli.board_number);
    let global_trace_filename = timestamped_filename(
        &global_trace_filename_prefix,
        config
            .test_gen
            .trace_file_suffix
            .clone()
            .to_str()
            .expect("Invalid trace file suffix"),
    );

    let global_trace_log_path = trace_dir_path.join(global_trace_filename);
    let global_trace_log_file = File::create(&global_trace_log_path)
        .with_context(|| format!("Creating global trace log file: {}", global_trace_log_path.display()))?;
    let global_trace_log = BufWriter::new(global_trace_log_file);

    let global_error_filename_prefix = format!("error_{}", cli.board_number);
    let global_error_filename = timestamped_filename(
        &global_error_filename_prefix,
        config
            .test_gen
            .error_file_suffix
            .clone()
            .to_str()
            .expect("Invalid error file suffix"),
    );

    let global_error_log_path = trace_dir_path.join(global_error_filename);
    let global_error_log_file = File::create(&global_error_log_path)
        .with_context(|| format!("Creating global error log file: {}", global_error_log_path.display()))?;
    let global_error_log = BufWriter::new(global_error_log_file);

    let (load_register_buffer, store_register_buffer) = match config.test_gen.cpu_type {
        MooCpuType::Intel80286 => (Cursor::new(vec![0; 102]), vec![0; 102]),
        MooCpuType::Intel80386Ex => (Cursor::new(vec![0; 204]), vec![0; 208]),
        _ => {
            eprintln!("Unsupported CPU type: {:?}", config.test_gen.cpu_type);
            std::process::exit(1);
        }
    };

    // Set some iced-x86 decoder options. We want to decode invalid forms where possible,
    // although iced-x86 is limited in the number of invalid forms it supports.
    let mut iced_decoder_opts = DecoderOptions::NO_INVALID_CHECK;
    // Enable CPU-specific decoding if needed.
    match config.test_gen.cpu_type {
        MooCpuType::Intel80286 => {
            iced_decoder_opts |= DecoderOptions::LOADALL286;
        }
        MooCpuType::Intel80386Ex => {
            iced_decoder_opts |= DecoderOptions::LOADALL386;
        }
        _ => {}
    }

    if config.test_gen.exclude_esc_opcodes {
        config
            .test_gen
            .excluded_opcodes
            .extend(config.test_gen.esc_opcodes.clone());
    }

    // Create the weighted index distribution
    let weights: Vec<f32> = config.test_gen.inject_values.iter().map(|v| v.weight).collect();
    let weighted_index = WeightedIndex::new(&weights)?;
    let inject_values: Vec<u32> = config.test_gen.inject_values.iter().map(|v| v.value).collect();

    let mut ctx = TestContext {
        exec_mode: ExecMode::default(),
        client: cpu_client,
        cfg: config,
        job_ct: cli.num_boards,
        job_no: cli.board_number,
        isa_db,
        load_register_buffer,
        store_register_buffer,
        server_cpu,
        register_set_type: RegisterSetType::from(server_cpu),
        test_opcode_size_prefix: TestOpcodeSizePrefix::None,
        code_segment_size: SegmentSize::Sixteen,
        file_seed: 0,
        prefetch: false,
        gen_start: Instant::now(),
        gen_stop: Instant::now(),
        gen_ct: 0,
        gen_total: 0,
        file_gen_ct: 0,
        output_path: output_dir_path,
        trace_path: trace_dir_path,
        validate_output_path: validate_output_dir,
        trace_log,
        global_trace_log,
        global_error_log,
        iced_decoder_opts,
        dry_run: cli.dry_run,
        last_program_state: None,
        weighted_index,
        inject_values,
        stats: Default::default(),
    };

    if cli.validate {
        ctx.exec_mode = ExecMode::Validate;
        validate::validate_tests::validate_tests(&mut ctx)?;
    }
    else {
        generate::gen_tests::generate_tests(&mut ctx)?;
    }

    Ok(())
}
