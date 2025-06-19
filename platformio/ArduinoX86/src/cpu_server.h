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

#ifndef _CPU_SERVER_H
#define _CPU_SERVER_H

// Protocol version number.
#define VERSION_NUM ((uint8_t)3)

// States for main program state machine:
// ----------------------------------------------------------------------------
// Reset - CPU is being reset
// JumpVector - CPU is jumping from reset vector to load segment (optional?)
// Load - CPU is executing register Load program
// LoadDone - CPU has finished executing Load program and waiting for program execution to start
// Execute - CPU is executing user program
// Store - CPU has is executing register Store program
typedef enum {
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
  Done
} machine_state_t;

typedef enum {
  CmdNone            = 0x00,
  CmdVersion         = 0x01,
  CmdReset           = 0x02,
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
  CmdCycleGetCycleState = 0x15,
  CmdPrefetchStore   = 0x16,
  CmdReadAddress     = 0x17,
  CmdCpuType         = 0x18,
  CmdEmulate8080     = 0x19,
  CmdPrefetch        = 0x1A,
  CmdInitScreen      = 0x1B,
  CmdInvalid         = 0x1C,
} server_command;

typedef bool (*command_func)();

#define RESPONSE_FAIL 0x00
#define RESPONSE_OK 0x01

// List of valid arguments to CmdWritePin. Only these specific pins
// can have state written to.
const uint8_t WRITE_PINS[] = {
  6,  // READY
  7,  // TEST
  12, // INTR
  13, // NMI
};

// Number of argument bytes expected for each command
const uint8_t CMD_INPUTS[] = {
  0,  // CmdNone
  0,  // CmdVersion
  0,  // CmdReset
  28, // CmdLoad
  0,  // CmdCycle
  0,  // CmdReadAddressLatch
  0,  // CmdReadStatus
  0,  // CmdRead8288Command 
  0,  // CmdRead8288Control 
  0,  // CmdReadDataBus 
  2,  // CmdWriteDataBus
  0,  // CmdFinalize
  0,  // CmdBeginStore,
  0,  // CmdStore,
  0,  // CmdQueueLen,
  0,  // CmdQueueBytes,
  2,  // CmdWritePin,
  1,  // CmdReadPin,
  0,  // CmdGetProgramState,
  0,  // CmdGetLastError,
  0,  // CmdGetCycleState,
  0,  // CmdCycleGetCycleState,
  0,  // CmdPrefetchStore,
  0,  // CmdReadAddress
  0,  // CmdCpuType
  0,  // CmdEmulate8080
  0,  // CmdPrefetch
  0,  // CmdInitScreen
  0,  // CmdInvalid
};

typedef enum {
  WaitingForCommand = 0x01,
  ReadingCommand,
  ExecutingCommand
} command_state_t;

typedef struct server_state {
  command_state_t c_state;
  server_command cmd;
  uint8_t cmd_byte_n;
  uint8_t cmd_bytes_expected;
  uint32_t cmd_start_time;
} CpuServer;

bool cmd_version(void);
bool cmd_reset(void);
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
bool cmd_cycle_get_cycle_state(void);
bool cmd_prefetch_store(void);
bool cmd_read_address(void);
bool cmd_cpu_type(void);
bool cmd_invalid(void);
bool cmd_emu8080(void);
bool cmd_prefetch(void);
bool cmd_init_screen(void);

#endif
