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

use std::{io::BufWriter, time::Instant};

use crate::{
    batch::TestCandidateList,
    generate::{gen_test::generate_consistent_test, generation_stats::GenerationStats},
    global_trace_log,
    trace_banner,
    trace_error,
    trace_flush,
    trace_log,
    TestContext,
};

use anyhow::{bail, Context};
use arduinox86_client::ServerFlags;
use marty_dasm::Opcode;
use moo::{
    prelude::*,
    types::{MooCpuType, MooFileMetadata},
};

/// The top-level test generation function.
/// Iterates through all opcodes specified by the configuration, generating tests for each one
/// until the specified test count is reached or an unrecoverable error occurs.
pub fn generate_tests(ctx: &mut TestContext) -> anyhow::Result<()> {
    ctx.gen_ct = 0;
    ctx.gen_start = Instant::now();

    let config = ctx.cfg.clone();

    // Set the opcode range to generate as specified by the configuration.
    // `opcode_range` is now a two-element array so safe to index directly.
    let mut opcode_range_start = config.test_gen.opcode_range[0];
    let mut opcode_range_end = config.test_gen.opcode_range[1];

    if let Some(opcode_override) = config.test_gen.opcode_override {
        opcode_range_start = opcode_override;
        opcode_range_end = opcode_override;
    }

    println!(
        "Generating tests for opcodes from [{} to {}]",
        Opcode::from(opcode_range_start),
        Opcode::from(opcode_range_end)
    );

    // Tell ArduinoX86 to execute instructions automatically.
    let mut server_flags = ServerFlags::EXECUTE_AUTOMATIC | ServerFlags::ENABLE_CYCLE_LOGGING;

    // Enable SMM register loading if we have a 386 CPU.
    // We can load registers as we exit SMM between tests - this avoids the lengthy 386EX setup
    // routine to disable wait states.
    if let MooCpuType::Intel80386Ex = config.test_gen.cpu_type {
        server_flags |= ServerFlags::USE_SMM;
    }

    ctx.client.set_flags(server_flags)?;
    ctx.client.enable_debug(config.test_exec.serial_debug_default)?;

    let mut generation_job = TestCandidateList::collect(ctx);
    if config.test_gen.skip_validated {
        generation_job.filter_validated(ctx);
    }
    if generation_job.is_empty() {
        bail!("No work to do based on filtering and batch criteria.");
    }
    generation_job.log(ctx);
    ctx.gen_total = generation_job.test_ct(ctx);

    let mut last_opcode: u16 = generation_job.iter().next().unwrap().opcode.into();

    for test_gen in generation_job.iter() {
        ctx.file_gen_ct = 0;
        ctx.stats = GenerationStats::default();
        ctx.stats.mnemonic_set.clear();
        ctx.test_opcode_size_prefix = test_gen.size_prefix;

        let opcode_raw: u16 = test_gen.opcode.into();
        last_opcode = opcode_raw;

        // Create the output file path.
        let mut file_path = ctx.output_path.clone();
        let filename = test_gen.filename();
        file_path.push(filename.clone());

        // Create the trace file.
        let trace_filename = test_gen.trace_filename(&config.test_gen.trace_file_suffix);
        let trace_file_path = ctx.trace_path.join(trace_filename);
        let trace_file = match config.test_gen.append_file {
            true => std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&trace_file_path)
                .with_context(|| format!("Opening trace file: {}", trace_file_path.display()))?,
            false => std::fs::File::create(&trace_file_path)
                .with_context(|| format!("Creating trace file: {}", trace_file_path.display()))?,
        };

        ctx.trace_log = BufWriter::new(trace_file);

        // Create the file seed by combining the opcode and extension then XORing with the base seed.
        let mut file_seed: u64 = opcode_raw as u64;
        let opcode_ext = test_gen.opcode_extension.unwrap_or(0);
        file_seed <<= 3;
        file_seed |= (opcode_ext & 0x07) as u64;
        file_seed ^= config.test_gen.base_seed;

        ctx.file_seed = file_seed;
        let mut test_start_num = 0;

        // Capture the CPU type from the client for the test file.
        let moo_arch = MooCpuType::from(ctx.client.cpu_type()?.0);

        // Create the new empty MooTestFile to hold our generated tests.
        let mut test_file = MooTestFile::new(
            config.test_gen.moo_version_major,
            config.test_gen.moo_version_minor,
            moo_arch,
            config.test_gen.test_count,
        );

        // Create a new MooFileMetadata for this test file.
        let mut test_metadata = MooFileMetadata::new(
            config.test_gen.set_version_major,
            config.test_gen.set_version_minor,
            config.test_gen.cpu_type.into(),
            opcode_raw as u32,
            test_gen.opcode_extension,
        )
        .with_file_seed(ctx.file_seed);

        // Open existing files if append == true
        if config.test_gen.append_file {
            // Open `filename` for reading as a BufReader.
            match std::fs::File::open(&file_path) {
                Ok(file) => {
                    log::debug!("Appending to existing test file: {}", file_path.to_string_lossy());
                    global_trace_log!(ctx, "Appending to existing test file: {}", file_path.to_string_lossy());
                    let mut file_reader = std::io::BufReader::new(file);
                    test_file = MooTestFile::read(&mut file_reader)?;

                    println!(
                        "Read {} tests from existing file: {}",
                        test_file.test_ct(),
                        file_path.to_string_lossy()
                    );

                    test_start_num = test_file.test_ct();
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        // If the file does not exist, we will create it later.

                        let prefix = match ctx.dry_run {
                            true => "would",
                            false => "will",
                        };
                        let debug_str = format!(
                            "File {} not found, {} create new test file.",
                            file_path.to_string_lossy(),
                            prefix,
                        );
                        log::debug!("{}", &debug_str);
                        global_trace_log!(ctx, "{}", debug_str);
                    }
                    else {
                        return Err(anyhow::anyhow!("Error opening test file: {}", e));
                    }
                }
            }
        };

        // Get the test count for this opcode, looking up any test count overrides.
        // We override the test count for certain trivial opcodes.
        // Nobody needs 10,000 tests of INC, after all.
        let test_count = config.test_gen.get_test_count(test_gen.opcode);

        // Skip test files that are already complete.
        if test_start_num >= test_count {
            println!("Test file {} is complete. Skipping...", file_path.to_string_lossy());
            ctx.gen_ct += test_count;
            continue;
        }

        // Generate the actual tests.
        for test_num in test_start_num..test_count {
            // `generate_consistent_test` checks for consistency - that is it generates a
            // test `validate_count` times and ensures the results are the same each time.
            // This helps catch and reject errors caused by intermittent hardware faults.
            let test_result = generate_consistent_test(
                ctx,
                test_num,
                test_gen.opcode,
                test_gen.opcode_extension,
                config.test_exec.validate_count as usize,
            );

            if !ctx.dry_run {
                // If test generation failed, log the error and return.
                // This is a fatal error as `generate_consistent_test` has built-in retries,
                // and we must have exhausted the retry threshold.
                if test_result.is_err() {
                    let err_msg = format!(
                        "Failed to generate test for opcode {} at test number {}: {}",
                        test_gen.opcode,
                        test_num,
                        test_result.as_ref().err().unwrap()
                    );
                    trace_error!(ctx, "{}", err_msg);
                    return Err(anyhow::anyhow!(err_msg));
                }

                // Add the test to the test file.
                let test = test_result?;
                test_file.add_test(test);
                ctx.file_gen_ct += 1;
            }
            ctx.gen_ct += 1;
        }
        // Test generation is complete.

        // Validate stats.
        if ctx.stats.mnemonic_set.len() != 1 {
            let err_msg = format!(
                "Error: Expected exactly one mnemonic for opcode {:02X}, found {}: {:?}",
                opcode_raw,
                ctx.stats.mnemonic_set.len(),
                ctx.stats.mnemonic_set
            );
            trace_error!(ctx, "{}", err_msg);
            log::error!("{}", err_msg);

            if ctx.cfg.test_gen.stop_on_error {
                return Err(anyhow::anyhow!(err_msg));
            }
        }

        // Log time taken
        ctx.gen_stop = Instant::now();
        if config.test_exec.show_gen_time {
            let gen_duration = ctx.gen_stop.duration_since(ctx.gen_start);
            println!(
                "Generated {} tests in {:.2?} seconds ({} tests per second)",
                ctx.gen_ct,
                gen_duration,
                ctx.gen_ct as f64 / gen_duration.as_secs_f64()
            );
        }

        trace_banner!(ctx);
        trace_log!(
            ctx,
            "### Test generation complete for opcode {} ({} tests) ###",
            opcode_raw,
            ctx.file_gen_ct
        );

        // Adjust final metadata with count...
        test_metadata = test_metadata.with_test_count((test_start_num as u32) + ctx.gen_ct as u32);
        // ... and with the most frequently seen mnemonic (to handle some tests that have invalid forms icedx86 won't decode).
        if let Some((mnemonic, _count)) = ctx.stats.most_frequent_mnemonic() {
            test_metadata = test_metadata.with_mnemonic(mnemonic.to_string());
        }

        // Log the generation stats.

        // ctx owns stats, so we can't pass the stats::log method the ctx to use
        // with our trace_log macro, so be lazy and just clone it out.
        let stats = ctx.stats.clone();
        stats.log(ctx);

        test_file.set_metadata(test_metadata);

        if !ctx.dry_run {
            // Open the file as a Writer.
            log::debug!("Writing test file: {}", file_path.to_string_lossy());

            let file = std::fs::File::create(&file_path)?;
            let mut writer = BufWriter::new(file);

            test_file.write(&mut writer, false)?;
        }

        trace_flush!(ctx);
    }

    println!("Test generation complete at terminating opcode: {:02X}", last_opcode);

    Ok(())
}
