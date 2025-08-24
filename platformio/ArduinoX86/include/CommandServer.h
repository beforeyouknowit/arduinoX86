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

#pragma once

#include <array>
#include <cstdint>
#include <cstdarg>
#include <serial_config.h>
//#include <BoardController.h>
#include <config.h>

enum class ServerState: uint8_t {
  Reset           = 0x00, // The CPU is being reset. Also used as initial state.
  CpuId           = 0x01, // The CPU ID routine is being executed. 
  CpuSetup        = 0x02, // The CPU setup routine is being executed. This is used for 186/386EX. 
  JumpVector      = 0x03, // The Jump Vector routine is being executed. This is used to avoid address wrap during Load.
  Load            = 0x04, // The register load routine is being executed. 
  LoadSmm         = 0x05, // The register load routine is being executed via SMM (386EX)
  LoadDone        = 0x06,
  EmuEnter        = 0x07,
  Prefetch        = 0x08,
  Execute         = 0x09,
  ExecuteFinalize = 0x0A,
  ExecuteDone     = 0x0B,
  EmuExit         = 0x0C,
  Store           = 0x0D,
  StoreDone       = 0x0E, // Store has completed. The CPU may need to be reset at this point if STOREALL was executed.
  StoreDoneSmm    = 0x0F, // Store has completed with SMM enabled. We remain in SMM and can Load registers as we exit.
  Done            = 0x10,
  StoreAll        = 0x11, // STOREALL (or SMM on 386) is in progress.
  Shutdown        = 0x12, // The CPU has shutdown (286/386). The CPU will need to be reset to continue.
  Error
};

template<typename BoardType, typename HatType> class BoardController;

template<typename BoardType, typename HatType>
class CommandServer {
public:

  using CmdFn = bool (CommandServer::*)();

  static constexpr uint32_t FLAG_EMU_8080           = 0x00000001;
  static constexpr uint32_t FLAG_EXECUTE_AUTOMATIC  = 0x00000002;
  static constexpr uint32_t FLAG_MEMORY_BACKEND     = 0x00000004; // 0=SDRAM, 1=Hash Table
  static constexpr uint32_t FLAG_HALT_AFTER_JUMP    = 0x00000008; // Halt after flow control instruction.
  static constexpr uint32_t FLAG_USE_SDRAM_BACKEND  = 0x00000010; // Use SDRAM as memory backend (requires GIGA)
  static constexpr uint32_t FLAG_USE_SMM            = 0x00000020; // Use SMM for register readout on 386/486 CPUs
  static constexpr uint32_t FLAG_DEBUG_ENABLED      = 0x00000040; // Enable debug mode

  enum class ServerCommand {
    CmdNone            = 0x00,
    CmdVersion         = 0x01,
    CmdResetCpu        = 0x02,
    CmdLoad            = 0x03,
    CmdCycle           = 0x04,
    CmdReadAddressLatch= 0x05,
    CmdReadStatus      = 0x06,
    CmdRead8288Command = 0x07,
    CmdRead8288Control = 0x08, 
    CmdReadDataBus     = 0x09,
    CmdWriteDataBus    = 0x0A,
    CmdFinalize        = 0x0B,
    CmdBeginStore      = 0x0C,
    CmdStore           = 0x0D,
    CmdQueueLen        = 0x0E,
    CmdQueueBytes      = 0x0F,
    CmdWritePin        = 0x10,
    CmdReadPin         = 0x11,
    CmdGetProgramState = 0x12,
    CmdLastError       = 0x13,
    CmdGetCycleState   = 0x14,
    CmdAvailable00     = 0x15,
    CmdPrefetchStore   = 0x16,
    CmdReadAddress     = 0x17,
    CmdCpuType         = 0x18,
    CmdSetFlags        = 0x19,
    CmdPrefetch        = 0x1A,
    CmdInitScreen      = 0x1B,
    CmdStoreAll        = 0x1C,
    CmdSetRandomSeed   = 0x1D,
    CmdRandomizeMem    = 0x1E,
    CmdSetMemory       = 0x1F,
    CmdGetCycleStates  = 0x20,
    CmdEnableDebug     = 0x21,
    CmdSetMemoryStrategy = 0x22,
    CmdGetFlags        = 0x23,
    CmdReadMemory      = 0x24,
    CmdEraseMemory     = 0x25,
    CmdServerStatus    = 0x26,
    CmdClearCycleLog   = 0x27,
    CmdInvalid
  };

  enum class CommandState: uint8_t {
    WaitingForCommand = 0x01,
    ReadingCommand,
    ExecutingCommand
  };

  void reset();
  void run();
  void change_state(ServerState new_state);
  ServerState state() const { return state_; }
  const char* get_last_error() const;

  const char* get_command_name(ServerCommand cmd);
  const char* get_state_string(ServerState state);
  char get_state_char(ServerState state);
  ServerState get_state() const {
    return state_;
  }
  uint32_t get_flags() const {
    return flags_;
  }

  bool is_execute_automatic() const {
    return (flags_ & FLAG_EXECUTE_AUTOMATIC) != 0;
  }

  bool halt_after_jump() const {
    return (flags_ & FLAG_HALT_AFTER_JUMP) != 0;
  }

  explicit CommandServer(BoardController<BoardType,HatType>& controller);

private:
  static constexpr std::size_t CMD_COUNT =
    static_cast<std::size_t>(ServerCommand::CmdInvalid);
  std::array<CmdFn, CMD_COUNT> commands_;

  static constexpr uint8_t VERSION_NUM = 3;
  static constexpr uint8_t RESPONSE_FAIL = 0x00;
  static constexpr uint8_t RESPONSE_OK = 0x01;
  static constexpr size_t MAX_COMMAND_BYTES = 255; // Maximum number of bytes for fixed-length parameters.
  static constexpr unsigned long CMD_TIMEOUT_ = CMD_TIMEOUT; // Timeout for command parameter bytes in milliseconds.
  uint8_t commandBuffer_[MAX_COMMAND_BYTES];

  BoardController<BoardType,HatType>& controller_;
  ServerState state_ = ServerState::Reset;
  CommandState commandState_ = CommandState::WaitingForCommand;
  uint8_t commandByte_ = 0;
  ServerCommand cmd_ = ServerCommand::CmdNone;
  bool useSmm_ = false; // Use SM mode for register readout on 386/486 CPUs
  size_t commandBytesExpected_ = 0;
  size_t commandByteN_ = 0;
  unsigned long commandStartTime_ = 0;
  unsigned long stateBeginTime_ = 0;
  uint32_t flags_ = 0; 

  // Error handling
  static constexpr size_t MAX_ERROR_LEN = 256;
  char errorBuffer_[MAX_ERROR_LEN] = {0};

  bool dispatch_command(ServerCommand cmd);
  uint8_t get_command_input_bytes(ServerCommand cmd);

  // Error handling methods
  void set_error(const char* format, ...);
  void clear_error() {
    set_error("NO ERROR");
  }

  void proto_write(const uint8_t* buf, size_t len) {
    INBAND_SERIAL.write(buf, len);
  }

  void proto_write(uint8_t b) {
    INBAND_SERIAL.write(b);
  }

  void proto_flush() {
    FLUSH;
  }

  int proto_available() {
    return INBAND_SERIAL.available();
  }

  int proto_read() {
    return INBAND_SERIAL.read();
  }

  size_t proto_read(uint8_t* buf, size_t len) {
    return INBAND_SERIAL.readBytes(buf, len);
  }

  int proto_peek() {
    return INBAND_SERIAL.peek();
  }

  void send_ok() {
    proto_write(RESPONSE_OK);
    proto_flush();
  }

  void send_fail() {
    proto_write(RESPONSE_FAIL);
    proto_flush();
  }

  void debug_cmd(const char *msg) {
    controller_.getBoard().debugPrintf(DebugType::CMD, false, "## cmd: %s ##\n\r", msg);
  }

  void debug_proto(const char* msg) {
    controller_.getBoard().debugPrintf(DebugType::PROTO, false, "## proto: %s ##\n\r", msg);
  }

  bool cmd_version(void);
  bool cmd_reset_cpu(void);
  bool cmd_load(void);
  bool cmd_cycle(void);
  bool cmd_read_address_latch(void);
  bool cmd_read_status(void);
  bool cmd_read_8288_command(void);
  bool cmd_read_8288_control(void);
  bool cmd_read_data_bus(void);
  bool cmd_write_data_bus(void);
  bool cmd_finalize(void);
  bool cmd_begin_store(void);
  bool cmd_store(void);
  bool cmd_queue_len(void);
  bool cmd_queue_bytes(void);
  bool cmd_write_pin(void);
  bool cmd_read_pin(void);
  bool cmd_get_program_state(void);
  bool cmd_get_last_error(void);
  bool cmd_get_cycle_state(void);
  bool cmd_prefetch_store(void);
  bool cmd_read_address(void);
  bool cmd_cpu_type(void);
  bool cmd_invalid(void);
  bool cmd_set_flags(void);
  bool cmd_prefetch(void);
  bool cmd_init_screen(void);
  bool cmd_storeall(void);
  bool cmd_set_random_seed(void);
  bool cmd_randomize_mem(void);
  bool cmd_set_memory(void);
  bool cmd_get_cycle_states(void);
  bool cmd_enable_debug(void);
  bool cmd_set_memory_strategy(void);
  bool cmd_get_flags(void);
  bool cmd_read_memory(void);
  bool cmd_erase_memory(void);
  bool cmd_server_status(void);
  bool cmd_clear_cycle_log(void);
  bool cmd_null(void);
};