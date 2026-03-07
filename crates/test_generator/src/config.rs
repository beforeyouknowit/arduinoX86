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
    enums::{InstructionSize, TerminationCondition},
    CpuMode,
};
use marty_dasm::Opcode;
use moo::types::MooCpuType;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct TestMetadata {
    pub repo: String,
    pub version: String,
    pub syntax_version: u32,
    pub cpu: String,
    pub cpu_detail: String,
    pub generator: String,
    pub author: String,
    pub date: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CountOverride {
    pub count: usize,
    pub opcode_range: [u16; 2],
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupExtensionOverride {
    pub opcode: u16,
    pub group_extension_range: [u8; 2],
}

#[derive(Clone, Debug, Deserialize)]
pub struct StackPointerOverride {
    pub opcode: u16,
    pub odd_chance: f32,
    pub min: u32,
    pub max: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExceptionSieveEntry {
    pub opcode: u16,
    pub extension: u8,
    pub exception: u8,
    pub exception_rate: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExceptionOverride {
    pub opcode: u16,
    pub extension: u8,
    pub allow_all: bool,
    pub exceptions: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModRmOverride {
    pub opcode: u16,
    pub extension: u8,
    pub allow_reg_form: bool,
    pub mask: u8,
    pub invalid_chance: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub test_gen:  TestGen,
    pub test_exec: TestExec,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ValidateMode {
    pub attempts: u32,
    pub move_after_validate: bool,
    pub stop_on_error: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WeightedValue {
    pub value:  u32,
    pub weight: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TestExec {
    pub polling_sleep: u32,
    pub validate_count: u32,
    pub max_sieve: u32,
    pub max_gen: u32,
    pub test_retry: u32,
    pub load_retry: u32,
    pub test_timeout: u32,
    pub print_instruction: bool,
    pub print_initial_regs: bool,
    pub print_final_regs: bool,
    pub show_gen_time: bool,
    pub serial_timeout: u32,
    pub serial_debug_default: bool,
    pub serial_debug_test: Option<usize>,
    pub validate_mode: ValidateMode,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TestGen {
    pub set_version_major: u8,
    pub set_version_minor: u8,
    pub cpu_type: MooCpuType,
    pub cpu_mode: CpuMode,
    pub base_seed: u64,
    pub stop_on_error: bool,
    pub termination_condition: TerminationCondition,
    pub test_output_dir: PathBuf,
    pub trace_output_dir: PathBuf,
    pub validate_output_dir: PathBuf,
    pub skip_validated: bool,
    pub trace_file_suffix: PathBuf,
    pub error_file_suffix: PathBuf,
    pub moo_version_major: u8,
    pub moo_version_minor: u8,
    pub moo_arch: String,
    pub gen_widths: Vec<InstructionSize>,
    pub skip_io: bool,
    pub address_mask: u32,
    pub ip_mask: u16,
    pub instruction_address_range: [u32; 2],

    pub max_normal_cycles:  u32,
    pub max_normal_bus_ops: u32,

    pub opcode_range: [u16; 2],
    pub opcode_override: Option<u16>,
    pub group_extension_range: [u8; 2],
    pub isa_db: PathBuf,
    pub group_extension_overrides: Vec<GroupExtensionOverride>,

    pub valid_opcodes: Vec<u16>,
    pub excluded_opcodes: Vec<u16>,
    pub exclude_esc_opcodes: bool,

    pub test_count:  usize,
    pub append_file: bool,

    pub writeless_null_shifts: bool,
    pub shift_mask: u16,

    pub register_beta: [f64; 2],
    pub max_prefixes:  usize,
    pub prefix_beta:   [f64; 2],

    pub lock_prefix_chance: f32,
    pub lock_prefix_opcode: u8,
    pub rep_prefix_chance: f32,
    pub sib_chance: Option<f32>,

    pub reg_zero_chance: f32,
    pub reg_ones_chance: f32,
    pub reg_inject_chance: f32,
    pub imm_zero_chance: f32,
    pub imm_ones_chance: f32,
    pub imm_inject_chance: f32,
    pub imm8s_min_chance: f32,
    pub imm8s_max_chance: f32,
    pub imm8s_inject_chance: f32,

    pub inject_values: Vec<WeightedValue>,

    pub near_branch_ban: Vec<i32>,
    pub io_port_ban: Vec<u16>,

    pub sp_odd_chance: f32,
    pub sp_min_value: u32,
    pub sp_max_value: u32,
    pub sp_min_address: u32,
    pub instruction_pad: u32,
    pub mem_zero_chance: f32,
    pub mem_ones_chance: f32,
    pub mem_strategy_start: u32,
    pub mem_strategy_end: u32,

    pub extended_prefix: u8,
    pub operand_size_prefix: Option<u8>,
    pub address_size_prefix: Option<u8>,
    pub group_opcodes: Vec<u16>,
    pub protected_mode_opcodes: Vec<u16>,
    pub esc_opcodes: Vec<u16>,
    pub flow_control_opcodes: Vec<u16>,
    pub bitfield_opcodes: Vec<u16>,
    pub io_opcodes: Vec<u16>,
    pub offset_opcodes: Vec<u16>,
    pub ptr_opcodes: Vec<u16>,
    pub far_indirect_opcodes: Vec<u16>,
    pub prefixes: Vec<u8>,
    pub segment_prefixes: Vec<u8>,
    pub disable_operand_size_prefix: Vec<u16>,
    pub disable_address_size_prefix: Vec<u16>,
    pub rep_prefixes: Vec<u8>,
    pub rep_opcodes: Vec<u16>,
    pub rep_cx_mask: u16,

    pub disable_seg_overrides: Vec<u16>,
    pub disable_lock_prefix:   Vec<u16>,

    pub sp_overrides: Vec<StackPointerOverride>,
    pub modrm_overrides: Vec<ModRmOverride>,
    pub count_overrides: Vec<CountOverride>,
    pub allowed_exceptions: Vec<u8>,
    pub exception_overrides: Vec<ExceptionOverride>,
    pub exception_sieve: Vec<ExceptionSieveEntry>,

    pub randomize_mem_interval: usize,
}

impl TestGen {
    pub fn get_test_count(&self, opcode: Opcode) -> usize {
        for ct_override in &self.count_overrides {
            let [min, max] = &ct_override.opcode_range[..]
            else {
                continue;
            };
            let opcode_u16: u16 = opcode.into();
            if opcode_u16 >= *min && opcode_u16 <= *max {
                log::trace!("Using test count override for opcode {}: {}", opcode, ct_override.count);
                return std::cmp::min(self.test_count, ct_override.count);
            }
        }
        log::trace!("Using default test count of {}", self.test_count);
        self.test_count
    }

    pub fn get_group_extension_range(&self, opcode: Opcode) -> (u8, u8) {
        for ext_override in &self.group_extension_overrides {
            if ext_override.opcode == opcode.into() {
                return (
                    ext_override.group_extension_range[0],
                    ext_override.group_extension_range[1],
                );
            }
        }
        (self.group_extension_range[0], self.group_extension_range[1])
    }
}
