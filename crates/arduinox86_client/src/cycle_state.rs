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
use crate::{get_queue_op, BusState, CpuWidth, DataWidth, ProgramState, QueueOp, Segment, ServerCpuType, TState};
use std::{
    cell::Cell,
    fmt::{Display, Formatter},
};

#[derive(Clone, Debug)]
pub struct ServerCycleState {
    pub program_state: ProgramState,
    pub cpu_state_bits: u8,
    pub cpu_status_bits: u8,
    pub bus_control_bits: u8,
    pub bus_command_bits: u8,
    pub address_bus: u32,
    pub data_bus: u16,
    pub pins: u16,
}

impl ServerCycleState {
    pub const CONTROL_ALE_BIT: u8 = 0b0000_0001;

    pub const COMMAND_MRDC_BIT: u8 = 0b0000_0001;
    pub const COMMAND_AMWC_BIT: u8 = 0b0000_0010;
    pub const COMMAND_MWTC_BIT: u8 = 0b0000_0100;
    pub const COMMAND_IORC_BIT: u8 = 0b0000_1000;
    pub const COMMAND_AIOWC_BIT: u8 = 0b0001_0000;
    pub const COMMAND_IOWC_BIT: u8 = 0b0010_0000;
    pub const COMMAND_INTA_BIT: u8 = 0b0100_0000;
    pub const COMMAND_BHE_BIT: u8 = 0b1000_0000;

    pub const STATUS_SEG_BITS: u8 = 0b0001_1000;

    pub const PIN_ALE: u16 = 0b0000_0000_0000_0001;
    pub const PIN_BHE: u16 = 0b0000_0000_0000_0010;
    pub const PIN_READY: u16 = 0b0000_0000_0000_0100;
    pub const PIN_LOCK: u16 = 0b0000_0000_0000_1000;

    #[inline]
    pub fn bhe(&self) -> bool {
        // BHE is active low.
        (self.bus_command_bits & Self::COMMAND_BHE_BIT) == 0
    }
    #[inline]
    pub fn ale(&self) -> bool {
        (self.bus_control_bits & Self::CONTROL_ALE_BIT) != 0
    }
    #[inline]
    pub fn t_state(&self) -> TState {
        TState::try_from(self.cpu_state_bits & 0x07).unwrap_or(TState::Ti)
    }
    #[inline]
    pub fn segment(&self) -> Segment {
        Segment::from(self.cpu_status_bits >> 3)
    }
    #[inline]
    pub fn is_reading_mem(&self) -> bool {
        (self.bus_command_bits & Self::COMMAND_MRDC_BIT) == 0
    }
    #[inline]
    pub fn is_writing_mem(&self) -> bool {
        (self.bus_command_bits & Self::COMMAND_MWTC_BIT) == 0
    }
    #[inline]
    pub fn is_reading_io(&self) -> bool {
        (self.bus_command_bits & Self::COMMAND_IORC_BIT) == 0
    }
    #[inline]
    pub fn is_writing_io(&self) -> bool {
        (self.bus_command_bits & Self::COMMAND_IOWC_BIT) == 0
    }
    #[inline]
    pub fn is_reading(&self) -> bool {
        self.is_reading_mem() || self.is_reading_io()
    }
    #[inline]
    pub fn is_writing(&self) -> bool {
        self.is_writing_mem() || self.is_writing_io()
    }
}

pub struct ServerCycleStatePrinter {
    pub cpu_type: ServerCpuType,
    pub address_latch: u32,
    pub state: ServerCycleState,
}

impl ServerCycleStatePrinter {
    pub fn data_width(&self) -> DataWidth {
        let cpu_width = CpuWidth::from(self.cpu_type);
        match cpu_width {
            CpuWidth::Eight => DataWidth::EightLow,
            CpuWidth::Sixteen => {
                if (self.address_latch & 1 != 0)
                    && (self.state.bus_command_bits & ServerCycleState::COMMAND_BHE_BIT == 0)
                {
                    DataWidth::EightHigh
                }
                else if self.state.pins & ServerCycleState::PIN_BHE == 0 {
                    DataWidth::Sixteen
                }
                else {
                    DataWidth::EightLow
                }
            }
        }
    }

    pub fn data_bus_str(&self) -> String {
        match self.data_width() {
            DataWidth::Invalid => "----".to_string(),
            DataWidth::Sixteen => format!("{:04X}", self.state.data_bus),
            DataWidth::EightLow => format!("{:>4}", format!("{:02X}", self.state.data_bus as u8)),
            DataWidth::EightHigh => format!("{:<4}", format!("{:02X}", (self.state.data_bus >> 8) as u8)),
        }
    }
}

impl Display for ServerCycleStatePrinter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ale_str = match self.state.ale() {
            true => "A:",
            false => "  ",
        };

        let mut seg_str = "  ".to_string();
        if self.cpu_type.has_segment_status() && self.state.t_state() != TState::T1 {
            // Segment status only valid in T2+
            seg_str = self.state.segment().to_string();
        }

        let q_op = get_queue_op!(self.state.cpu_status_bits);
        let q_op_chr = match q_op {
            QueueOp::Idle => ' ',
            QueueOp::First => 'F',
            QueueOp::Flush => 'E',
            QueueOp::Subsequent => 'S',
        };

        // All read/write signals are active/low
        let rs_chr = match self.state.bus_command_bits & ServerCycleState::COMMAND_MRDC_BIT == 0 {
            true => 'R',
            false => '.',
        };
        let aws_chr = match self.state.bus_command_bits & 0b0000_0010 == 0 {
            true => 'A',
            false => '.',
        };
        let ws_chr = match self.state.bus_command_bits & 0b0000_0100 == 0 {
            true => 'W',
            false => '.',
        };
        let ior_chr = match self.state.bus_command_bits & 0b0000_1000 == 0 {
            true => 'R',
            false => '.',
        };
        let aiow_chr = match self.state.bus_command_bits & 0b0001_0000 == 0 {
            true => 'A',
            false => '.',
        };
        let iow_chr = match self.state.bus_command_bits & 0b0010_0000 == 0 {
            true => 'W',
            false => '.',
        };

        let bhe_chr = match self.state.bhe() {
            true => 'B',
            false => '.',
        };

        let intr_chr = if false { 'I' } else { '.' };
        let inta_chr = if self.state.bus_command_bits & ServerCycleState::COMMAND_INTA_BIT == 0 {
            '.'
        }
        else {
            '.'
        };

        let bus_state = self.cpu_type.decode_status(self.state.cpu_status_bits);
        let bus_raw = self.cpu_type.raw_status(self.state.cpu_status_bits);
        let bus_str = match bus_state {
            BusState::INTA => "INTA",
            BusState::IOR => "IOR ",
            BusState::IOW => "IOW ",
            BusState::HALT => "HALT",
            BusState::CODE => "CODE",
            BusState::MEMR => "MEMR",
            BusState::MEMW => "MEMW",
            BusState::PASV => "PASV",
        };

        let t_string = self.cpu_type.tstate_to_string(self.state.t_state());

        let mut xfer_str = "        ".to_string();

        let bus_active = match self.cpu_type {
            ServerCpuType::Intel80386 => {
                // The 386 can write on t1
                if self.state.is_writing() {
                    true
                }
                else {
                    // The 386 can read after T1
                    self.state.t_state() != TState::T1
                }
            }
            ServerCpuType::Intel80286 => {
                // The 286 can read/write after T1
                self.state.t_state() != TState::T1
            }
            _ => {
                // Older CPUs can only read/write in PASV state
                bus_state == BusState::PASV
            }
        };

        if bus_active {
            let value = self.data_bus_str();
            if self.state.is_reading() {
                xfer_str = format!("r-> {}", value);
            }
            else if self.state.is_writing() {
                xfer_str = format!("<-w {}", value);
            }
        }

        let bus_chr_width = self.cpu_type.bus_chr_width();
        let data_chr_width = self.cpu_type.data_chr_width();
        write!(
            f,
            "{ale_str:02}{addr_latch:0bus_chr_width$X}:{addr_bus:0bus_chr_width$X}:{data_bus:0data_chr_width$X} \
            {seg_str:02} M:{rs_chr}{aws_chr}{ws_chr} I:{ior_chr}{aiow_chr}{iow_chr} \
            P:{intr_chr}{inta_chr}{bhe_chr} {bus_str:04}[{bus_raw:01}] {t_str:02} {xfer_str:06}",
            ale_str = ale_str,
            addr_latch = self.address_latch,
            addr_bus = self.state.address_bus,
            data_bus = self.state.data_bus,
            bus_chr_width = bus_chr_width,
            seg_str = seg_str,
            rs_chr = rs_chr,
            aws_chr = aws_chr,
            ws_chr = ws_chr,
            ior_chr = ior_chr,
            aiow_chr = aiow_chr,
            iow_chr = iow_chr,
            intr_chr = intr_chr,
            inta_chr = inta_chr,
            bhe_chr = bhe_chr,
            bus_str = bus_str,
            bus_raw = bus_raw,
            t_str = t_string,
            xfer_str = xfer_str,
            // q_op_chr = q_op_chr,
            // q_str = self.queue.to_string(),
            // width = self.queue.size() * 2,
            // q_read_str = q_read_str,
        )
    }
}

pub struct ServerCycleLogPrinter<'a> {
    cpu_type: ServerCpuType,
    states: &'a [ServerCycleState],
    cycle_number: Cell<u64>,
    address_latch: Cell<u32>,
}

impl<'a> ServerCycleLogPrinter<'a> {
    pub fn new(cpu_type: ServerCpuType, states: &'a [ServerCycleState]) -> Self {
        Self {
            cpu_type,
            states,
            cycle_number: Cell::new(0),
            address_latch: Cell::new(0),
        }
    }
}

impl Display for ServerCycleLogPrinter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let format_count = self.states.len().saturating_sub(1);
        let cycle_digits = format_count.to_string().len();

        for state in self.states {
            if state.ale() {
                self.address_latch.set(state.address_bus);
            }

            let printer = ServerCycleStatePrinter {
                cpu_type: self.cpu_type,
                address_latch: self.address_latch.get(),
                state: state.clone(),
            };
            // Write with cycle_digits number padding
            writeln!(f, "{:0cycle_digits$} {}", self.cycle_number.get(), printer)?;
            self.cycle_number.set(self.cycle_number.get() + 1);
        }
        Ok(())
    }
}
