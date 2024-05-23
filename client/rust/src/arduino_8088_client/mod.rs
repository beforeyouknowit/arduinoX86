#![allow(dead_code, unused_variables)]
use std::io::{Read, Write};
use serialport::{SerialPort, ClearBuffer};
use log;

pub const ARD8088_BAUD: u32 = 1000000;
//pub const ARD8088_BAUD: u32 = 115200;

//pub const ARD8088_BAUD: u32 = 460800;

#[derive(Copy, Clone)]
pub enum ServerCommand {
                                
    CmdNull             = 0x00,
    CmdVersion          = 0x01,
    CmdReset            = 0x02,
    CmdLoad             = 0x03,
    CmdCycle            = 0x04,
    CmdReadAddressLatch = 0x05,
    CmdReadStatus       = 0x06,
    CmdRead8288Command  = 0x07,
    CmdRead8288Control  = 0x08, 
    CmdReadDataBus      = 0x09,
    CmdWriteDataBus     = 0x0A,
    CmdFinalize         = 0x0B,
    CmdBeginStore       = 0x0C,
    CmdStore            = 0x0D,
    CmdQueueLen         = 0x0E,
    CmdQueueBytes       = 0x0F,
    CmdWritePin         = 0x10,
    CmdReadPin          = 0x11,
    CmdGetProgramState  = 0x12,
    CmdGetLastError     = 0x13,
    CmdGetCycleState    = 0x14,
    CmdCGetCycleState   = 0x15,
    CmdPrefetchStore    = 0x16,
    CmdReadAddressU     = 0x17,
    CmdInvalid          = 0x18,
}

#[derive(Debug, PartialEq)]
pub enum ProgramState {
    Reset = 0,
    JumpVector,
    Load,
    LoadDone,
    Execute,
    ExecuteFinalize,
    ExecuteDone,
    Store,
    StoreDone,
    Done
}

#[derive (PartialEq)]
pub enum Segment {
    ES = 0,
    SS,
    CS,
    DS
}

#[derive (PartialEq)]
pub enum QueueOp {
    Idle = 0,
    First,
    Flush,
    Subsequent,
}

#[derive (PartialEq)]
pub enum BusState {
    INTA = 0,   // IRQ Acknowledge
    IOR  = 1,   // IO Read
    IOW  = 2,   // IO Write
    HALT = 3,   // Halt
    CODE = 4,   // Code
    MEMR = 5,   // Memory Read
    MEMW = 6,   // Memory Write
    PASV = 7    // Passive
}

#[derive (PartialEq)]
pub enum CpuPin {
    READY = 0,
    TEST,
    INTR,
    NMI
}

pub const REQUIRED_PROTOCOL_VER: u8 = 0x01;

pub const COMMAND_MRDC_BIT: u8  = 0b0000_0001;
pub const COMMAND_AMWC_BIT: u8  = 0b0000_0010;
pub const COMMAND_MWTC_BIT: u8  = 0b0000_0100;
pub const COMMAND_IORC_BIT: u8  = 0b0000_1000;
pub const COMMAND_AIOWC_BIT: u8 = 0b0001_0000;
pub const COMMAND_IOWC_BIT: u8  = 0b0010_0000;
pub const COMMAND_INTA_BIT: u8  = 0b0100_0000;
pub const COMMAND_ALE_BIT: u8   = 0b1000_0000;

pub const STATUS_SEG_BITS: u8   = 0b0001_1000;

macro_rules! get_segment {
    ($s:expr) => {
        match (($s >> 3) & 0x03) {
            0b00 => Segment::ES,
            0b01 => Segment::SS,
            0b10 => Segment::CS,
            _ => Segment::DS
        }
    };
}

macro_rules! get_bus_state {
    ($s:expr) => {
        match ($s & 0x07) {
            0 => BusState::INTA,
            1 => BusState::IOR,
            2 => BusState::IOW,
            3 => BusState::HALT,
            4 => BusState::CODE,
            5 => BusState::MEMR,
            6 => BusState::MEMW,
            _ => BusState::PASV
        }
    };
}

macro_rules! get_queue_op {
    ($s:expr) => {
        match (($s >> 6) & 0x03) {
            0b00 => QueueOp::Idle,
            0b01 => QueueOp::First,
            0b10 => QueueOp::Flush,
            _ => QueueOp::Subsequent
        }
    };
}

macro_rules! is_reading {
    ($s:expr) => {
        match ((!($s) & 0b0000_1001) != 0) {
            true => true,
            false => false
        }
    }
}

macro_rules! is_writing {
    ($s:expr) => {
        match ((!($s) & 0b0011_0110) != 0) {
            true => true,
            false => false
        }
    }
}

use std::{
    rc::Rc,
    cell::RefCell, 
    error::Error,
    fmt::Display,
    str,
};

#[derive (Debug)]
pub enum CpuClientError {
    ReadFailure,
    WriteFailure,
    BadValue,
    ReadTimeout,
    EnumerationError,
    DiscoveryError,
    CommandFailed,
}

impl Error for CpuClientError {}
impl Display for CpuClientError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            
            CpuClientError::ReadFailure => {
                write!(f, "Failed to read from serial port." )
            }
            CpuClientError::WriteFailure => {
                write!(f, "Failed to write to serial port.")
            }
            CpuClientError::BadValue => {
                write!(f, "Received invalid value from command.")
            }
            CpuClientError::ReadTimeout => {
                write!(f, "Response timeout.")
            }
            CpuClientError::EnumerationError => {
                write!(f, "Failed to find a valid serial port.")
            }
            CpuClientError::DiscoveryError => {
                write!(f, "Failed to find a listening Arduino8088 server.")
            }
            CpuClientError::CommandFailed => {
                write!(f, "Server command returned failure code.")
            }            
        }
    }
}

pub struct CpuClient {

    port: Rc<RefCell<Box<dyn serialport::SerialPort>>>,
}

impl CpuClient {
    pub fn init() -> Result<CpuClient, CpuClientError> {
        match serialport::available_ports() {
            Ok(ports) => {
                for port in ports {
                    log::debug!("Found serial port: {}", port.port_name );
                    if let Some(rtk_port) = CpuClient::try_port(port) {
                        return Ok(
                            CpuClient {
                                port: Rc::new(RefCell::new(rtk_port))
                            }
                        )
                    }
                }
            },
            Err(e) => {
                log::error!("Didn't find any serial ports: {:?}", e);
                return Err(CpuClientError::EnumerationError);
            }
        };
        Err(CpuClientError::DiscoveryError)
    }

    pub fn try_port(port_info: serialport::SerialPortInfo) -> Option<Box<dyn SerialPort>> {

        let port_result = serialport::new(port_info.port_name.clone(), 0)
            .baud_rate(0)
            .timeout(std::time::Duration::from_millis(100))
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
                log::trace!("Sending version query...");

                match new_port.write(&cmd) {
                    Ok(_) => {
                        log::trace!("Sent version query...");
                    },
                    Err(e) => {
                        log::error!("try_port: Write error: {:?}", e);
                        return None
                    }
                }
                match new_port.flush() {
                    Ok(_) => {
                        log::trace!("Flushed output...");
                    },
                    Err(e) => {
                        log::error!("try_port: flush error: {:?}", e);
                        return None
                    }
                }

                let bytes_read = match new_port.read(&mut buf) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        log::error!("try_port: Read error: {:?}", e);
                        return None;
                    }
                };

                new_port.clear(serialport::ClearBuffer::Input).unwrap();
                if bytes_read == 9 {
                    let ver_text = str::from_utf8(&buf).unwrap();
                    if ver_text.contains("ard8088") {

                        let proto_ver = buf[7];
                        log::trace!("Found an Arduino8088 server, protocol verison: {} on port {}", proto_ver, port_info.port_name);

                        if proto_ver != REQUIRED_PROTOCOL_VER {
                            log::error!("Unsupported protocol version.");
                            return None
                        }
                    }
                    return Some(new_port)
                }     
                else {
                    log::trace!("Invalid response from discovery command. Read {} bytes (Expected 9).", bytes_read);
                    let ver_text = str::from_utf8(&buf).unwrap();
                    log::trace!("First 9 bytes of response: {:?}", ver_text);
                }           
                None
            }
            Err(e) => {
                log::error!("try_port: Error opening host port: {}", e);
                None
            }
        }
    }
    
    pub fn send_command_byte(&mut self, cmd: ServerCommand) -> Result<(), CpuClientError> {
        let cmd: [u8; 1] = [cmd as u8];

        self.port.borrow_mut().clear(ClearBuffer::Input).unwrap();
        match self.port.borrow_mut().write(&cmd) {
            Ok(_) => {
                Ok(())
            },
            Err(_) => {
                Err(CpuClientError::WriteFailure)
            }
        }
    }

    pub fn read_result_code(&mut self) -> Result<bool, CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];

        match self.port.borrow_mut().read(&mut buf) {
            Ok(bytes) => {
                if bytes == 0 {
                    log::error!("read_result_code: 0 bytes read");
                    Err(CpuClientError::ReadFailure)
                }
                else if (buf[0] & 0x01) != 0 {
                    // LSB set in return code == success
                    Ok(true)
                }
                else {
                    log::error!("read_result_code: command returned failure");
                    Err(CpuClientError::CommandFailed)
                }
            },
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
                }
                else {
                    Ok(true)
                }
            },
            Err(_) => {
                Err(CpuClientError::WriteFailure)
            }
        }
    }

    pub fn recv_buf(&mut self, buf: &mut [u8]) -> Result<bool, CpuClientError> {
        match self.port.borrow_mut().read(buf) {
            Ok(bytes) => {
                if bytes != buf.len() {
                    // We didn't read entire buffer worth of data, fail
                    log::error!("recv_buf: Only read {} bytes of {}.", bytes, buf.len());
                    Err(CpuClientError::ReadFailure)
                }
                else {
                    Ok(true)
                }
                
            },
            Err(e) => {
                log::error!("recv_buf: read operation failed: {}", e);
                Err(CpuClientError::ReadFailure)
            }
        }
    }    

    /// Receive a buffer of dynamic size (don't expect the entire buffer read like recv_buf does)
    /// Returns the number of bytes read.
    /// Primarily used for get_last_error
    pub fn recv_dyn_buf(&mut self, buf: &mut [u8]) -> Result<usize, CpuClientError> {
        match self.port.borrow_mut().read(buf) {
            Ok(bytes) => {
                Ok(bytes)
            },
            Err(_) => {
                Err(CpuClientError::ReadFailure)
            }
        }
    }    

    /// Server command - Load
    /// Load the specified register state into the CPU.
    /// This command takes 28 bytes, which correspond to the word values of each of the 14
    /// CPU registers.
    /// Registers should be loaded in the following order, little-endian:
    ///
    /// AX, BX, CX, DX, SS, SP, FLAGS, IP, CS, DS, ES, BP, SI, DI    
    pub fn load_registers_from_buf(&mut self, reg_data: &[u8]) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdLoad)?;
        self.send_buf(reg_data)?;
        self.read_result_code()
    }

    pub fn begin_store(&mut self) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdBeginStore)?;
        self.read_result_code()
    }    

    pub fn store_registers_to_buf(&mut self, reg_data: &mut [u8]) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdStore)?;
        self.recv_buf(reg_data)?;
        self.read_result_code()
    }

    pub fn cycle(&mut self) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdCycle)?;
        self.read_result_code()
    }

    pub fn read_address_latch(&mut self) -> Result<u32, CpuClientError> {
        let mut buf: [u8; 3] = [0; 3];
        self.send_command_byte(ServerCommand::CmdReadAddressLatch)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code()?;

        let address = buf[0] as u32 | (buf[1] as u32) << 8 | (buf[2] as u32) << 16;

        Ok(address)
    }

    pub fn read_address(&mut self) -> Result<u32, CpuClientError> {
        let mut buf: [u8; 3] = [0; 3];
        self.send_command_byte(ServerCommand::CmdReadAddressU)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code()?;

        let address = buf[0] as u32 | (buf[1] as u32) << 8 | (buf[2] as u32) << 16;

        Ok(address)
    }

    pub fn read_status(&mut self) -> Result<u8, CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];
        self.send_command_byte(ServerCommand::CmdReadStatus)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code()?;

        Ok(buf[0])
    }

    pub fn read_8288_command(&mut self) -> Result<u8, CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];
        self.send_command_byte(ServerCommand::CmdRead8288Command)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code()?;

        Ok(buf[0])
    }

    pub fn read_8288_control(&mut self) -> Result<u8, CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];
        self.send_command_byte(ServerCommand::CmdRead8288Control)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code()?;

        Ok(buf[0])
    }

    pub fn read_data_bus(&mut self) -> Result<u8, CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];
        self.send_command_byte(ServerCommand::CmdReadDataBus)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code()?;

        Ok(buf[0])
    }
    
    pub fn write_data_bus(&mut self, data: u8) -> Result<bool, CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];
        self.send_command_byte(ServerCommand::CmdWriteDataBus)?;

        buf[0] = data;
        self.send_buf(&mut buf)?;
        self.read_result_code()?;

        Ok(true)
    }

    pub fn prefetch_store(&mut self) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdPrefetchStore)?;
        self.read_result_code()
    }

    pub fn finalize(&mut self) -> Result<bool, CpuClientError> {
        self.send_command_byte(ServerCommand::CmdFinalize)?;
        self.read_result_code()
    }

    pub fn get_program_state(&mut self) -> Result<ProgramState, CpuClientError> {
        let mut buf: [u8; 1] = [0; 1];
        self.send_command_byte(ServerCommand::CmdGetProgramState)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code()?;

        match buf[0] {
            0x00 => Ok(ProgramState::Reset),
            0x01 => Ok(ProgramState::JumpVector),
            0x02 => Ok(ProgramState::Load),
            0x03 => Ok(ProgramState::LoadDone),
            0x04 => Ok(ProgramState::Execute),
            0x05 => Ok(ProgramState::ExecuteFinalize),
            0x06 => Ok(ProgramState::ExecuteDone),
            0x07 => Ok(ProgramState::Store),
            0x08 => Ok(ProgramState::StoreDone),
            0x09 => Ok(ProgramState::Done),
            _ => Err(CpuClientError::BadValue)
        }
    }

    pub fn get_last_error(&mut self) -> Result<String, CpuClientError> {
        //let mut cmdbuf: [u8; 1] = [0; 1];
        let mut errbuf: [u8; 50] = [0; 50];
        self.send_command_byte(ServerCommand::CmdGetLastError)?;
        let bytes = self.recv_dyn_buf(&mut errbuf)?;
        let err_string = str::from_utf8(&errbuf[..bytes-1]).unwrap();

        Ok(err_string.to_string())
    }

    pub fn write_pin(&mut self, pin_no: CpuPin, val: bool) -> Result<bool, CpuClientError> {
        let mut buf: [u8; 2] = [0; 2];
        buf[0] = pin_no as u8;
        buf[1] = val as u8;
        self.send_command_byte(ServerCommand::CmdWritePin)?;
        self.send_buf(&mut buf)?;
        self.read_result_code()
    }
    
    pub fn get_cycle_state(&mut self) -> Result<(ProgramState, u8, u8, u8, u8), CpuClientError> {
        let mut buf: [u8; 4] = [0; 4];
        self.send_command_byte(ServerCommand::CmdGetCycleState)?;
        self.recv_buf(&mut buf)?;
        self.read_result_code()?;

        let state_bits: u8 = buf[0] >> 4;
        let state: ProgramState = match state_bits {
            0x00 => ProgramState::Reset,
            0x01 => ProgramState::JumpVector,
            0x02 => ProgramState::Load,
            0x03 => ProgramState::LoadDone,
            0x04 => ProgramState::Execute,
            0x05 => ProgramState::ExecuteFinalize,
            0x06 => ProgramState::ExecuteDone,
            0x07 => ProgramState::Store,
            0x08 => ProgramState::StoreDone,
            0x09 => ProgramState::Done,
            _ => {
                return Err(CpuClientError::BadValue);
            }
        };

        let control_bits = buf[0] & 0x0F;

        Ok((state, control_bits, buf[1], buf[2], buf[3]))
    }
}