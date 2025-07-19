#![allow(dead_code, unused_variables)]

mod registers;

use std::io::{Read, Write};
use std::{cell::RefCell, rc::Rc, str};

use log;
use serialport::{ClearBuffer, SerialPort};
use thiserror::Error;

pub const ARDUINO_BAUD: u32 = 1000000;
pub use registers::*;

pub struct ServerFlags;

#[derive(Copy, Clone, Debug)]
pub enum MemoryStrategy {
    Random,
    Zero,
    Ones,
}

#[rustfmt::skip]
impl ServerFlags {
    pub const EMU_8080: u32             = 0x0000_0001; // 8080 emulation enabled
    pub const EXECUTE_AUTOMATIC: u32    = 0x0000_0002; // Execute automatically after load
    pub const HASH_BACKEND: u32         = 0x0000_0004; // Use hash backend for memory
    pub const HALT_AFTER_JUMP: u32      = 0x0000_0008; // Insert halt after flow control
}

/// [ServerCommand] represents the commands that can be sent to the Arduino808X server.
#[derive(Copy, Clone, Debug)]
pub enum ServerCommand {
    CmdNull = 0x00,
    CmdVersion = 0x01,
    CmdReset = 0x02,
    CmdLoad = 0x03,
    CmdCycle = 0x04,
    CmdReadAddressLatch = 0x05,
    CmdReadStatus = 0x06,
    CmdRead8288Command = 0x07,
    CmdRead8288Control = 0x08,
    CmdReadDataBus = 0x09,
    CmdWriteDataBus = 0x0A,
    CmdFinalize = 0x0B,
    CmdBeginStore = 0x0C,
    CmdStore = 0x0D,
    CmdQueueLen = 0x0E,
    CmdQueueBytes = 0x0F,
    CmdWritePin = 0x10,
    CmdReadPin = 0x11,
    CmdGetProgramState = 0x12,
    CmdGetLastError = 0x13,
    CmdGetCycleState = 0x14,
    CmdNull2 = 0x15,
    CmdPrefetchStore = 0x16,
    CmdReadAddressU = 0x17,
    CmdCpuType = 0x18,
    CmdSetFlags = 0x19,
    CmdPrefetch = 0x1A,
    CmdInitScreen = 0x1B,
    CmdStoreAll = 0x1C,
    CmdSetRandomSeed = 0x1D,
    CmdRandomizeMemory = 0x1E,
    CmdSetMemory = 0x1F,
    CmdGetCycleStates = 0x20,
    CmdEnableDebug = 0x21,
    CmdSetMemoryStrategy = 0x22,
    CmdGetFlags = 0x23,
    CmdInvalid,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TState {
    Ti,
    T1,
    T2,
    T3,
    T4,
    Tw,
}

impl TryFrom<u8> for TState {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TState::Ti),
            1 => Ok(TState::T1),
            2 => Ok(TState::T2),
            3 => Ok(TState::T3),
            4 => Ok(TState::T4),
            5 => Ok(TState::Tw),
            _ => Err(format!("Invalid bus cycle value: {}", value)),
        }
    }
}

/// [ServerCpuType] maps to the CPU types that can be detected by the Arduino808X server.
#[derive(Copy, Clone, Debug, Default)]
pub enum ServerCpuType {
    #[default]
    Undetected,
    Intel8088,
    Intel8086,
    NecV20,
    NecV30,
    Intel80188(bool),
    Intel80186(bool),
    Intel80286,
}

impl ServerCpuType {
    /// Returns whether the CPU type is an Intel CPU.
    pub fn is_intel(&self) -> bool {
        match self {
            ServerCpuType::Intel8088
            | ServerCpuType::Intel8086
            | ServerCpuType::Intel80188(_)
            | ServerCpuType::Intel80186(_)
            | ServerCpuType::Intel80286 => true,
            _ => false,
        }
    }
    /// Returns whether we can prefetch the user program for this CPU type.
    /// Currently, all CPU types support prefetching.
    pub fn can_prefetch(&self) -> bool {
        true
    }
    /// Returns whether this CPU type supports 8080 emulation. Only the NEC V20 and V30
    /// support this.
    pub fn has_8080_emulation(&self) -> bool {
        match self {
            ServerCpuType::NecV20 | ServerCpuType::NecV30 => true,
            _ => false,
        }
    }

    pub fn tstate_to_string(&self, state: TState) -> String {
        match self {
            ServerCpuType::Intel80286 => match state {
                TState::Ti => "Ti".to_string(),
                TState::T1 => "Ts".to_string(),
                TState::T2 => "Tc".to_string(),
                TState::Tw => "Tw".to_string(),
                _ => "T?".to_string(),
            },
            _ => match state {
                TState::Ti => "Ti".to_string(),
                TState::T1 => "T1".to_string(),
                TState::T2 => "T2".to_string(),
                TState::T3 => "T3".to_string(),
                TState::T4 => "T4".to_string(),
                TState::Tw => "Tw".to_string(),
            },
        }
    }

    // Return true if the specified TState should trigger a read or write.
    // For 80286, we have very tight timings so we must write the bus in advance of T2/Tc.
    pub fn is_write_cycle(&self, state: TState) -> bool {
        match self {
            ServerCpuType::Intel80286 => matches!(state, TState::T1),
            _ => matches!(state, TState::T2),
        }
    }

    pub fn bus_chr_width(&self) -> usize {
        match self {
            ServerCpuType::Intel80286 => 6,
            _ => 5,
        }
    }

    pub fn decode_status(&self, status_byte: u8) -> BusState {
        match self {
            ServerCpuType::Intel80286 => match status_byte & 0x0F {
                0b0000 => BusState::INTA,
                0b0001 => BusState::PASV, // Reserved
                0b0010 => BusState::PASV, // Reserved
                0b0011 => BusState::PASV, // None
                0b0100 => BusState::HALT,
                0b0101 => BusState::MEMR,
                0b0110 => BusState::MEMW,
                0b0111 => BusState::PASV, // None
                0b1000 => BusState::PASV, // Reserved
                0b1001 => BusState::IOR,
                0b1010 => BusState::IOW,
                0b1011 => BusState::PASV, // None
                0b1100 => BusState::PASV, // Reserved
                0b1101 => BusState::CODE,
                0b1110 => BusState::PASV, // Reserved
                0b1111 => BusState::PASV, // None
                _ => BusState::PASV,      // Default to passive state
            },
            _ => {
                match status_byte & 0x07 {
                    0 => BusState::INTA, // IRQ Acknowledge
                    1 => BusState::IOR,  // IO Read
                    2 => BusState::IOW,  // IO Write
                    3 => BusState::HALT, // Halt
                    4 => BusState::CODE, // Code fetch
                    5 => BusState::MEMR, // Memory Read
                    6 => BusState::MEMW, // Memory Write
                    _ => BusState::PASV, // Passive state
                }
            }
        }
    }
}

/// Derive the [CpuWidth] from a [ServerCpuType].
impl From<ServerCpuType> for CpuWidth {
    fn from(cpu_type: ServerCpuType) -> Self {
        match cpu_type {
            ServerCpuType::NecV20 | ServerCpuType::Intel8088 | ServerCpuType::Intel80188(_) => {
                CpuWidth::Eight
            }
            _ => CpuWidth::Sixteen,
        }
    }
}

/// Convert a raw u8 value received from the Arduino808X server to a [ServerCpuType].
impl TryFrom<u8> for ServerCpuType {
    type Error = CpuClientError;
    fn try_from(value: u8) -> Result<ServerCpuType, CpuClientError> {
        match value & 0x3F {
            0x00 => Ok(ServerCpuType::Undetected),
            0x01 => Ok(ServerCpuType::Intel8088),
            0x02 => Ok(ServerCpuType::Intel8086),
            0x03 => Ok(ServerCpuType::NecV20),
            0x04 => Ok(ServerCpuType::NecV30),
            0x05 => Ok(ServerCpuType::Intel80188((value & 0x80) != 0)),
            0x06 => Ok(ServerCpuType::Intel80186((value & 0x80) != 0)),
            0x07 => Ok(ServerCpuType::Intel80286),
            _ => Err(CpuClientError::TypeConversionError),
        }
    }
}

/// [DataWidth] represents the current width of the data bus.
#[derive(Copy, Clone, Debug, Default)]
pub enum DataWidth {
    #[default]
    Invalid,
    /// The entire data bus is being driven.
    Sixteen,
    /// The low half of the data bus is being driven, A0 is even.
    EightLow,
    /// The low half of the data bus is being driven, A0 is odd.
    EightHigh,
}

/// Convert the BHE and A0 signals to a [DataWidth].
impl From<(bool, bool)> for DataWidth {
    fn from(signals: (bool, bool)) -> DataWidth {
        match signals {
            (true, true) => {
                // BHE is enabled, A0 is odd. Eight bit read high half of bus is active.
                DataWidth::EightHigh
            }
            (true, false) => {
                // BHE is enabled, A0 is even. Sixteen bit read, full bus active.
                DataWidth::Sixteen
            }
            (false, true) => {
                // BHE is disabled, A0 is odd. This is an invalid condition -
                // neither half of the data bus is driven
                DataWidth::Invalid
            }
            (false, false) => {
                // BHE is disabled, A0 is even. Eight bit low half of bus is active.
                DataWidth::EightLow
            }
        }
    }
}

/// [CpuWidth] represents the width of the detected CPU's data bus.
#[derive(Copy, Clone, Debug, Default)]
pub enum CpuWidth {
    #[default]
    Eight,
    Sixteen,
}

/// Returns the size of the instruction queue for the CPU width.
impl CpuWidth {
    pub fn queue_size(&self) -> usize {
        match self {
            CpuWidth::Eight => 4,
            CpuWidth::Sixteen => 6,
        }
    }
}

/// Convert a raw u8 value received from the Arduino808X server to a [CpuWidth].
impl From<u8> for CpuWidth {
    fn from(value: u8) -> Self {
        match value {
            0 => CpuWidth::Eight,
            _ => CpuWidth::Sixteen,
        }
    }
}

/// Convert a [CpuWidth] to a usize value representing the number of bytes.
impl From<CpuWidth> for usize {
    fn from(value: CpuWidth) -> usize {
        match value {
            CpuWidth::Eight => 1,
            CpuWidth::Sixteen => 2,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub enum RegisterSetType {
    #[default]
    Intel8088,
    Intel286,
}

impl From<ServerCpuType> for RegisterSetType {
    fn from(cpu_type: ServerCpuType) -> Self {
        match cpu_type {
            ServerCpuType::Intel80286 => RegisterSetType::Intel286,
            _ => RegisterSetType::Intel8088,
        }
    }
}

impl From<u8> for RegisterSetType {
    fn from(value: u8) -> Self {
        match value {
            0 => RegisterSetType::Intel8088,
            1 => RegisterSetType::Intel286,
            _ => RegisterSetType::Intel8088, // Default to Intel8088 if unknown
        }
    }
}

impl From<RegisterSetType> for u8 {
    fn from(value: RegisterSetType) -> u8 {
        match value {
            RegisterSetType::Intel8088 => 0,
            RegisterSetType::Intel286 => 1,
        }
    }
}

/// [ProgramState] represents the current state of the Arduino808X server.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ProgramState {
    Reset = 0,
    CpuId,
    CpuSetup,
    JumpVector,
    Load,
    LoadDone,
    EmuEnter,
    Prefetch,
    Execute,
    ExecuteFinalize,
    ExecuteDone,
    EmuExit,
    Store,
    StoreDone,
    Done,
    StoreAll,
    Shutdown,
    Error,
}

/// Convert a raw u8 value received from the Arduino808X server to a [ProgramState].
impl TryFrom<u8> for ProgramState {
    type Error = CpuClientError;
    fn try_from(value: u8) -> Result<ProgramState, CpuClientError> {
        match value {
            0x00 => Ok(ProgramState::Reset),
            0x01 => Ok(ProgramState::CpuId),
            0x02 => Ok(ProgramState::CpuSetup),
            0x03 => Ok(ProgramState::JumpVector),
            0x04 => Ok(ProgramState::Load),
            0x05 => Ok(ProgramState::LoadDone),
            0x06 => Ok(ProgramState::EmuEnter),
            0x07 => Ok(ProgramState::Prefetch),
            0x08 => Ok(ProgramState::Execute),
            0x09 => Ok(ProgramState::ExecuteFinalize),
            0x0A => Ok(ProgramState::ExecuteDone),
            0x0B => Ok(ProgramState::EmuExit),
            0x0C => Ok(ProgramState::Store),
            0x0D => Ok(ProgramState::StoreDone),
            0x0E => Ok(ProgramState::Done),
            0x0F => Ok(ProgramState::StoreAll),
            0x10 => Ok(ProgramState::Shutdown),
            0x11 => Ok(ProgramState::Error),
            _ => Err(CpuClientError::TypeConversionError),
        }
    }
}

/// [Segment] represents the segment registers in the CPU.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Segment {
    ES = 0,
    SS,
    CS,
    DS,
}

/// [QueueOp] represents the operation performed on the instruction queue on the last cycle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum QueueOp {
    Idle = 0,
    First,
    Flush,
    Subsequent,
}

/// [BusState] represents the current state of the bus as decoded by the CPU S0-S2 status lines.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BusState {
    INTA = 0, // IRQ Acknowledge
    IOR = 1,  // IO Read
    IOW = 2,  // IO Write
    HALT = 3, // Halt
    CODE = 4, // Code
    MEMR = 5, // Memory Read
    MEMW = 6, // Memory Write
    PASV = 7, // Passive
}

/// [CpuPin] represents the miscellaneous CPU pins that can be read or written to.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CpuPin {
    READY = 0,
    TEST,
    INTR,
    NMI,
}

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

pub const REQUIRED_PROTOCOL_VER: u8 = 3;

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

#[macro_export]
macro_rules! get_segment {
    ($s:expr) => {
        match (($s >> 3) & 0x03) {
            0b00 => Segment::ES,
            0b01 => Segment::SS,
            0b10 => Segment::CS,
            _ => Segment::DS,
        }
    };
}

#[macro_export]
macro_rules! get_queue_op {
    ($s:expr) => {
        match (($s >> 6) & 0x03) {
            0b00 => QueueOp::Idle,
            0b01 => QueueOp::First,
            0b10 => QueueOp::Flush,
            _ => QueueOp::Subsequent,
        }
    };
}

#[macro_export]
macro_rules! is_reading {
    ($s:expr) => {
        match ((!($s) & 0b0000_1001) != 0) {
            true => true,
            false => false,
        }
    };
}

#[macro_export]
macro_rules! is_writing {
    ($s:expr) => {
        match ((!($s) & 0b0011_0110) != 0) {
            true => true,
            false => false,
        }
    };
}

/// [CpuClientError] represents the errors that can occur when communicating with the Arduino808X
/// server.
#[derive(Error, Debug)]
pub enum CpuClientError {
    #[error("Failed to read from serial port.")]
    ReadFailure,
    #[error("Failed to write to serial port.")]
    WriteFailure,
    #[error("Type conversion failed.")]
    TypeConversionError,
    #[error("{0:?} command received invalid value from server.")]
    BadValue(ServerCommand),
    #[error("Command was given an invalid parameter: {0}")]
    BadParameter(String),
    #[error("Response timeout waiting for command.")]
    ReadTimeout,
    #[error("Failed to enumerate serial ports.")]
    EnumerationError,
    #[error("Failed to find a listening ArduinoX86 server.")]
    DiscoveryError,
    #[error("{0:?} command returned failure code.")]
    CommandFailed(ServerCommand),
}

/// A [CpuClient] represents a connection to an `ArduinoX86` server over a serial port.
pub struct CpuClient {
    port: Rc<RefCell<Box<dyn serialport::SerialPort>>>,
}

impl CpuClient {
    pub fn init(
        com_port: Option<String>,
        timeout: Option<u64>,
    ) -> Result<CpuClient, CpuClientError> {
        let mut matched_port = false;
        match serialport::available_ports() {
            Ok(ports) => {
                for port in ports {
                    if let Some(ref p) = com_port {
                        if port.port_name != *p {
                            continue; // Skip ports that don't match the specified port
                        }
                        matched_port = true;
                    }
                    println!("Trying port: {}", port.port_name);
                    if let Some(rtk_port) = CpuClient::try_port(port, timeout.unwrap_or(1000)) {
                        return Ok(CpuClient {
                            port: Rc::new(RefCell::new(rtk_port)),
                        });
                    }
                }

                if let Some(ref p) = com_port {
                    return if !matched_port {
                        log::warn!("Did not find specified port: {}", p);
                        Err(CpuClientError::DiscoveryError)
                    } else {
                        log::warn!("Did not find Arduino808X server at specified port: {}", p);
                        Err(CpuClientError::DiscoveryError)
                    };
                }
                Err(CpuClientError::DiscoveryError)
            }
            Err(e) => {
                log::warn!("Didn't find any serial ports: {:?}", e);
                Err(CpuClientError::EnumerationError)
            }
        }
    }

    /// Try to open the specified serial port and query it for an Arduino808X server.
    pub fn try_port(
        port_info: serialport::SerialPortInfo,
        timeout: u64,
    ) -> Option<Box<dyn SerialPort>> {
        let port_result = serialport::new(port_info.port_name.clone(), 0)
            .baud_rate(0)
            .timeout(std::time::Duration::from_millis(timeout))
            .stop_bits(serialport::StopBits::One)
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .open();

        match port_result {
            Ok(mut new_port) => {
                //log::trace!("Successfully opened host port {}", port_info.port_name);

                // Flush
                new_port.clear(ClearBuffer::Input).unwrap();
                new_port.clear(ClearBuffer::Output).unwrap();

                let cmd: [u8; 1] = [1];
                let mut buf: [u8; 100] = [0; 100];

                _ = new_port.flush();
                log::trace!("Sending version query to {}...", port_info.port_name);

                match new_port.write(&cmd) {
                    Ok(_) => {
                        log::trace!("Sent version query to {}...", port_info.port_name);
                    }
                    Err(e) => {
                        log::error!("try_port: Write error to {}: {:?}", port_info.port_name, e);
                        return None;
                    }
                }
                match new_port.flush() {
                    Ok(_) => {
                        log::trace!("Flushed output to {}...", port_info.port_name);
                    }
                    Err(e) => {
                        log::error!(
                            "try_port: flush error from {}: {:?}",
                            port_info.port_name,
                            e
                        );
                        return None;
                    }
                }

                match new_port.read_exact(&mut buf[..9]) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        log::error!("try_port: Read error from {}: {:?}", port_info.port_name, e);
                        return None;
                    }
                };

                new_port.clear(serialport::ClearBuffer::Input).unwrap();
                let ver_text = str::from_utf8(&buf).unwrap();
                if ver_text.contains("ardX86 ") {
                    let proto_ver = buf[7];
                    log::trace!(
                        "Found an ArduinoX86 server, protocol verison: {} on port {}",
                        proto_ver,
                        port_info.port_name
                    );

                    if proto_ver != REQUIRED_PROTOCOL_VER {
                        log::error!("Unsupported protocol version.");
                        return None;
                    }
                }
                Some(new_port)
            }
            Err(e) => {
                log::error!(
                    "try_port: Error opening host port {}: {}",
                    port_info.port_name,
                    e
                );
                None
            }
        }
    }

    pub fn send_command_byte(&mut self, cmd: ServerCommand) -> Result<(), CpuClientError> {
        let cmd: [u8; 1] = [cmd as u8];
        let mut flush_buf: [u8; 100] = [0; 100];
        let mut port = self.port.borrow_mut();
        port.clear(ClearBuffer::Input).unwrap();
        if port.bytes_to_read().unwrap() > 0 {
            let _flushed_bytes = port.read(&mut flush_buf).unwrap();
        }

        match port.write(&cmd) {
            Ok(_) => Ok(()),
            Err(_) => Err(CpuClientError::WriteFailure),
        }
    }

    pub fn read_result_code(&mut self, cmd: ServerCommand) -> Result<bool, CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];

        match self.port.borrow_mut().read_exact(&mut buf) {
            Ok(()) => {
                if (buf[0] & 0x01) != 0 {
                    // LSB set in return code == success
                    Ok(true)
                } else {
                    log::error!("read_result_code: command returned failure: {:02X}", buf[0]);
                    Err(CpuClientError::CommandFailed(cmd))
                }
            }
            Err(e) => {
                log::error!("read_result_code: read operation failed: {}", e);
                Err(CpuClientError::ReadFailure)
            }
        }
    }

    pub fn send_buf(&mut self, buf: &[u8]) -> Result<bool, CpuClientError> {
        match self.port.borrow_mut().write(&buf) {
            Ok(bytes) => {
                if bytes != buf.len() {
                    Err(CpuClientError::WriteFailure)
                } else {
                    Ok(true)
                }
            }
            Err(_) => Err(CpuClientError::WriteFailure),
        }
    }

    pub fn recv_buf(&mut self, buf: &mut [u8]) -> Result<bool, CpuClientError> {
        self.port
            .borrow_mut()
            .read_exact(buf)
            .map_err(|_| CpuClientError::ReadFailure)
            .and_then(|_| {
                if buf.len() == 0 {
                    Err(CpuClientError::ReadFailure)
                } else {
                    Ok(true)
                }
            })
    }

    /// Receive a buffer of dynamic size (don't expect the entire buffer read like recv_buf does)
    /// Returns the number of bytes read.
    /// Primarily used for get_last_error
    pub fn recv_dyn_buf(&mut self, buf: &mut [u8]) -> Result<usize, CpuClientError> {
        match self.port.borrow_mut().read(buf) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(CpuClientError::ReadFailure),
        }
    }

    /// Server command - Load
    /// Load the specified register state into the CPU.
    /// This command takes 28 bytes, which correspond to the word values of each of the 14
    /// CPU registers.
    /// Registers should be loaded in the following order, little-endian:
    ///
    /// AX, BX, CX, DX, SS, SP, FLAGS, IP, CS, DS, ES, BP, SI, DI
    pub fn load_registers_from_buf(
        &mut self,
        reg_type: RegisterSetType,
        reg_data: &[u8],
    ) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdLoad)?;
        let mut buf: [u8; 1] = [0; 1];
        buf[0] = reg_type.into();
        self.send_buf(&buf)?;
        self.send_buf(reg_data)?;
        self.read_result_code(ServerCommand::CmdLoad)
    }

    pub fn begin_store(&mut self) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdBeginStore)?;
        self.read_result_code(ServerCommand::CmdBeginStore)
    }

    pub fn store_registers_to_buf(&mut self, reg_data: &mut [u8]) -> Result<u8, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdStore)?;
        let mut buf: [u8; 1] = [0; 1];
        self.recv_buf(&mut buf)?;

        match buf[0] {
            0 => {
                // Type 1 register set (Intel808X)
                if reg_data.len() < 28 {
                    return Err(CpuClientError::BadParameter(
                        "Expected at least 28 bytes for Intel808X register set".to_string(),
                    ));
                }
            }
            1 => {
                // Type 2 register set (Intel286)
                if reg_data.len() < 102 {
                    return Err(CpuClientError::BadParameter(
                        "Expected at least 102 bytes for Intel286 register set".to_string(),
                    ));
                }
            }
            _ => {
                // invalid register set type
                return Err(CpuClientError::BadValue(ServerCommand::CmdStore));
            }
        }
        self.recv_buf(reg_data)?;
        self.read_result_code(ServerCommand::CmdStore)?;

        Ok(buf[0])
    }

    pub fn cycle(&mut self) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdCycle)?;
        self.read_result_code(ServerCommand::CmdCycle)
    }

    pub fn cpu_type(&mut self) -> Result<(ServerCpuType, bool), CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];
        self.send_command_byte(ServerCommand::CmdCpuType)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code(ServerCommand::CmdCpuType)?;

        let cpu_type = ServerCpuType::try_from(buf[0])?;
        Ok((cpu_type, buf[0] & 0x40 != 0))
    }

    pub fn init_screen(&mut self) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdInitScreen)?;
        let mut buf: [u8; 1] = [0; 1];
        self.recv_buf(&mut buf)?;
        self.read_result_code(ServerCommand::CmdInitScreen)?;

        Ok(buf[0] != 0)
    }

    pub fn read_address_latch(&mut self) -> Result<u32, CpuClientError> {
        let mut buf: [u8; 3] = [0; 3];
        self.send_command_byte(ServerCommand::CmdReadAddressLatch)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code(ServerCommand::CmdReadAddressLatch)?;

        let address = buf[0] as u32 | (buf[1] as u32) << 8 | (buf[2] as u32) << 16;

        Ok(address)
    }

    pub fn read_address(&mut self) -> Result<u32, CpuClientError> {
        let mut buf: [u8; 3] = [0; 3];
        self.send_command_byte(ServerCommand::CmdReadAddressU)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code(ServerCommand::CmdReadAddressU)?;

        let address = buf[0] as u32 | (buf[1] as u32) << 8 | (buf[2] as u32) << 16;

        Ok(address)
    }

    pub fn read_status(&mut self) -> Result<u8, CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];
        self.send_command_byte(ServerCommand::CmdReadStatus)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code(ServerCommand::CmdReadStatus)?;

        Ok(buf[0])
    }

    pub fn read_8288_command(&mut self) -> Result<u8, CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];
        self.send_command_byte(ServerCommand::CmdRead8288Command)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code(ServerCommand::CmdRead8288Command)?;

        Ok(buf[0])
    }

    pub fn read_8288_control(&mut self) -> Result<u8, CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];
        self.send_command_byte(ServerCommand::CmdRead8288Control)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code(ServerCommand::CmdRead8288Control)?;

        Ok(buf[0])
    }

    pub fn read_data_bus(&mut self) -> Result<u16, CpuClientError> {
        let mut buf: [u8; 2] = [0; 2];
        self.send_command_byte(ServerCommand::CmdReadDataBus)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code(ServerCommand::CmdReadDataBus)?;

        let word = u16::from_le_bytes([buf[0], buf[1]]);
        Ok(word)
    }

    pub fn write_data_bus(&mut self, data: u16) -> Result<bool, CpuClientError> {
        let mut buf: [u8; 2] = [0; 2];
        self.send_command_byte(ServerCommand::CmdWriteDataBus)?;
        buf.copy_from_slice(&data.to_le_bytes());

        self.send_buf(&mut buf)?;
        self.read_result_code(ServerCommand::CmdWriteDataBus)?;

        Ok(true)
    }

    pub fn prefetch_store(&mut self) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdPrefetchStore)?;
        self.read_result_code(ServerCommand::CmdPrefetchStore)
    }

    pub fn finalize(&mut self) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdFinalize)?;
        self.read_result_code(ServerCommand::CmdFinalize)
    }

    pub fn get_program_state(&mut self) -> Result<ProgramState, CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];
        self.send_command_byte(ServerCommand::CmdGetProgramState)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code(ServerCommand::CmdGetProgramState)?;

        ProgramState::try_from(buf[0])
    }

    pub fn get_last_error(&mut self) -> Result<String, CpuClientError> {
        let mut errbuf: [u8; 50] = [0; 50];
        self.send_command_byte(ServerCommand::CmdGetLastError)?;
        let bytes = self.recv_dyn_buf(&mut errbuf)?;
        let err_string = str::from_utf8(&errbuf[..bytes - 1]).unwrap();

        Ok(err_string.to_string())
    }

    pub fn write_pin(&mut self, pin_no: CpuPin, val: bool) -> Result<bool, CpuClientError> {
        let mut buf: [u8; 2] = [0; 2];
        buf[0] = pin_no as u8;
        buf[1] = val as u8;
        self.send_command_byte(ServerCommand::CmdWritePin)?;
        self.send_buf(&mut buf)?;
        self.read_result_code(ServerCommand::CmdWritePin)
    }

    /// Get the per-cycle state of the CPU.
    /// Arguments:
    ///   `cycle`: If true, instruct the server to cycle the CPU once before returning the state.
    /// Returns:
    ///     A [ServerCycleState] struct containing the current state of the CPU.
    pub fn get_cycle_state(&mut self, cycle: bool) -> Result<ServerCycleState, CpuClientError> {
        let mut send_buf: [u8; 1] = [0; 1];
        if cycle {
            send_buf[0] = 1;
        }
        let mut recv_buf: [u8; 11] = [0; 11];
        self.send_command_byte(ServerCommand::CmdGetCycleState)?;
        self.send_buf(&mut send_buf)?;
        self.recv_buf(&mut recv_buf)?;
        self.read_result_code(ServerCommand::CmdGetCycleState)?;

        let cycle_state = ServerCycleState {
            program_state: ProgramState::try_from(recv_buf[0])?,
            cpu_state_bits: recv_buf[1],
            cpu_status_bits: recv_buf[2],
            bus_control_bits: recv_buf[3],
            bus_command_bits: recv_buf[4],
            address_bus: u32::from_le_bytes([recv_buf[5], recv_buf[6], recv_buf[7], recv_buf[8]]),
            data_bus: u16::from_le_bytes([recv_buf[9], recv_buf[10]]),
            pins: 0,
        };

        //log::trace!("received buffer: {:0X?}", recv_buf);

        Ok(cycle_state)
    }

    pub fn set_flags(&mut self, flags: u32) -> Result<bool, CpuClientError> {
        let buf: [u8; 4] = flags.to_le_bytes();
        self.send_command_byte(ServerCommand::CmdSetFlags)?;
        self.send_buf(&buf)?;
        self.read_result_code(ServerCommand::CmdSetFlags)
    }

    pub fn get_flags(&mut self) -> Result<u32, CpuClientError> {
        let mut buf: [u8; 4] = [0; 4];
        self.send_command_byte(ServerCommand::CmdGetFlags)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code(ServerCommand::CmdGetFlags)?;

        Ok(u32::from_le_bytes(buf))
    }

    pub fn storeall(&mut self) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdStoreAll)?;
        self.read_result_code(ServerCommand::CmdStoreAll)
    }

    pub fn randomize_memory(&mut self, seed: u32) -> Result<bool, CpuClientError> {
        let buf: [u8; 4] = seed.to_le_bytes();
        self.send_command_byte(ServerCommand::CmdRandomizeMemory)?;
        self.send_buf(&buf)?;
        self.read_result_code(ServerCommand::CmdRandomizeMemory)
    }

    pub fn set_random_seed(&mut self, seed: u32) -> Result<bool, CpuClientError> {
        let buf: [u8; 4] = seed.to_le_bytes();
        self.send_command_byte(ServerCommand::CmdSetRandomSeed)?;
        self.send_buf(&buf)?;
        self.read_result_code(ServerCommand::CmdSetRandomSeed)
    }

    pub fn set_memory(&mut self, address: u32, data_buf: &[u8]) -> Result<bool, CpuClientError> {
        log::trace!(
            "set_memory(): uploading {} bytes to address 0x{:08X}",
            data_buf.len(),
            address
        );
        let mut buf: [u8; 4] = address.to_le_bytes();
        let data_buf_len = data_buf.len() as u32;
        if data_buf_len == 0 {
            return Err(CpuClientError::BadParameter(
                "Data buffer cannot be empty".to_string(),
            ));
        }
        self.send_command_byte(ServerCommand::CmdSetMemory)?;
        // Send address
        self.send_buf(&mut buf)?;
        // Send size
        buf = data_buf_len.to_le_bytes();
        self.send_buf(&mut buf)?;
        // Send data
        self.send_buf(data_buf)?;
        self.read_result_code(ServerCommand::CmdSetMemory)
    }

    pub fn get_cycle_states(&mut self) -> Result<Vec<ServerCycleState>, CpuClientError> {
        let mut param_buf: [u8; 8] = [0; 8];

        self.send_command_byte(ServerCommand::CmdGetCycleStates)?;
        // We are guaranteed to have at least 8 bytes in the buffer, a count and a size
        self.recv_buf(&mut param_buf)?;
        let cycle_count =
            u32::from_le_bytes([param_buf[0], param_buf[1], param_buf[2], param_buf[3]]);
        let data_size =
            u32::from_le_bytes([param_buf[4], param_buf[5], param_buf[6], param_buf[7]]);

        let struct_size = data_size / cycle_count;
        let mut receive_buf = vec![0; data_size as usize];
        self.recv_buf(&mut receive_buf)?;

        let mut cycles: Vec<ServerCycleState> = Vec::with_capacity(cycle_count as usize);

        for i in 0..cycle_count {
            let offset = (i * struct_size) as usize;
            if offset + struct_size as usize > receive_buf.len() {
                return Err(CpuClientError::ReadFailure);
            }
            let cycle_data = &receive_buf[offset..offset + struct_size as usize];
            if cycle_data.len() < 12 {
                return Err(CpuClientError::ReadFailure);
            }
            let cycle_state = ServerCycleState {
                program_state: ProgramState::Execute,
                address_bus: u32::from_le_bytes([
                    cycle_data[0],
                    cycle_data[1],
                    cycle_data[2],
                    cycle_data[3],
                ]),
                data_bus: u16::from_le_bytes([cycle_data[4], cycle_data[5]]),
                cpu_state_bits: cycle_data[6],
                cpu_status_bits: cycle_data[7],
                bus_control_bits: cycle_data[8],
                bus_command_bits: cycle_data[9],
                pins: u16::from_le_bytes([cycle_data[10], cycle_data[11]]), // Skip pins [10][11]
            };
            cycles.push(cycle_state);
        }

        self.read_result_code(ServerCommand::CmdGetCycleStates)?;

        Ok(cycles)
    }

    pub fn set_memory_strategy(
        &mut self,
        strategy: MemoryStrategy,
        start: u32,
        end: u32,
    ) -> Result<bool, CpuClientError> {
        let mut buf: [u8; 9] = [0; 9];
        buf[0] = strategy as u8;
        buf[1..5].copy_from_slice(&start.to_le_bytes());
        buf[5..9].copy_from_slice(&end.to_le_bytes());

        self.send_command_byte(ServerCommand::CmdSetMemoryStrategy)?;
        self.send_buf(&buf)?;
        self.read_result_code(ServerCommand::CmdSetMemoryStrategy)
    }

    pub fn enable_debug(&mut self, enable: bool) -> Result<(), CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];
        buf[0] = if enable { 1 } else { 0 };
        self.send_command_byte(ServerCommand::CmdEnableDebug)?;
        self.send_buf(&buf)?;
        self.read_result_code(ServerCommand::CmdEnableDebug)?;
        Ok(())
    }
}
