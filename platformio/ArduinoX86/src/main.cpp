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

// This is the main code module for the "CPU Server" sketch that controls
// the CPU and establishes a serial protocol for a CPU client to set up
// and communicate with the CPU after initialization.



#include <Arduino.h>
#include <stdint.h>
#include "arduinoX86.h"
#include <globals.h>
#include "cpu_server.h"
#include "opcodes.h"
#include "Display.h"

#include <BoardController.h>

#ifdef GIGA_DISPLAY_SHIELD
#include "Arduino_GigaDisplay_GFX.h"
#include "GigaDisplay.h"
#endif 

CpuServer SERVER;
Cpu CPU;
Intel8288 I8288;

// Global pointer to abstract Display interface
Display* screen = nullptr;

// Timing stuff.

unsigned long frame_ms_accumulator = 0;
unsigned long second_ms_accumulator = 0;
unsigned long last_millis = 0;
unsigned int fps_counter = 0;

uint8_t SETUP_PROGRAM_86[] = { 0x90 };  // Not used

uint8_t SETUP_PROGRAM_186[] = {
  0xb8, 0x00, 0x00, 0xba, 0x18, 0xff, 0xef,  // MOV AX, 0 | MOV DX, FF18 | OUT DX, AX  ; Unmask Int0
  0xEA, 0x00, 0x00, 0x00, 0x00,              // FAR JUMP to [patched segment:0000]
};

#define SETUP_PATCH_VECTOR_OFFSET_186 10

// Register load routine. This program gets patched with the client supplied register values.
// It uses MOVs and POPs to set the register state as specified before the main program execution
// begins.
uint8_t LOAD_PROGRAM[] = {
  0x00, 0x00,
  0xB8, 0x00, 0x00, 0x8E, 0xD0, 0x89, 0xC4, 0x9D, 0xBB, 0x00, 0x00, 0xB9, 0x00, 0x00,
  0xBA, 0x00, 0x00, 0xB8, 0x00, 0x00, 0x8E, 0xD0, 0xB8, 0x00, 0x00, 0x8E, 0xD8, 0xB8, 0x00, 0x00,
  0x8E, 0xC0, 0xB8, 0x00, 0x00, 0x89, 0xC4, 0xB8, 0x00, 0x00, 0x89, 0xC5, 0xB8, 0x00, 0x00, 0x89,
  0xC6, 0xB8, 0x00, 0x00, 0x89, 0xC7, 0xB8, 0x00, 0x00, 0xEA, 0x00, 0x00, 0x00, 0x00
};

uint8_t LOAD_PROGRAM_286[] = {
  0x0F, 0x05, // LOADALL
};

// Patch offsets for load program
const size_t LOAD_BX = 0x0B;
const size_t LOAD_CX = 0x0E;
const size_t LOAD_DX = 0x11;
const size_t LOAD_SS = 0x14;
const size_t LOAD_DS = 0x19;
const size_t LOAD_ES = 0x1E;
const size_t LOAD_SP = 0x23;
const size_t LOAD_BP = 0x28;
const size_t LOAD_SI = 0x2D;
const size_t LOAD_DI = 0x32;
const size_t LOAD_AX = 0x37;
const size_t LOAD_IP = 0x3A;
const size_t LOAD_CS = 0x3C;

// CPU/FPU ID program.


// FPU detection is performed by issuing a `fnstcw` instruction followed by wait.
// If a write of 0x03FF is detected, then a FPU is present.
//
// CPU detection is pretty simple - Intel CPUs have the undocumented and very fast instruction
// SALC at D6 - NEC CPUs have an undefined alias for XLAT that takes a lot longer. We can simply
// measure the execution time to determine Intel vs NEC.
// This routine is run first, in the reset vector, before the Jump program.
const uint8_t CPUID_PROGRAM[] = {
  0xD6,                    // SALC/Undefined
  0xD9, 0x3E, 0x00, 0x00,  // fnstcw [0000]
  0x90,                    // wait
  0x90, 0x90,              // NOPs to absorb fetch while RQ/GT runs
};

// 8080 Emulation enter program. This program executes the BRKEM opcode to enter 8080 emulation
// on a compatible NEC CPU such as the V20 or V30.
// The first four bytes are used as the BRKEM vector segment and offset, and are patched with the
// values of CS and IP.
uint8_t EMU_ENTER_PROGRAM[] = {
  0x00, 0x00, 0x00, 0x00, 0x0F, 0xFF, BRKEM_VECTOR
};

// 8080 Emulation exit program. This program executes PUSH PSW to preseve the 8080 flag state,
// then POP PSW to restore BP, then executes RETEM to exit emulation mode.
const uint8_t EMU_EXIT_PROGRAM[] = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // 6 NOPs to hide program from client
  0xF5, 0x00,                          // PUSH PSW, NOP
  0x33, 0x33,                          // INX SP, INX SP to restore 8080 stack pointer
  0xED, 0xFD,                          // RETEM
};

// Far Jump program. We feed this program to the CPU at the reset vector. On an 8088 the reset
// vector is at FFFF:0000 or address FFFF0 - giving us only 16 bytes to the end of the address
// space, where we will wrap around. We could wrap, but it gets a bit confusing, so instead
// we'll jump to a clean new segment. The exact segment is configurable with LOAD_SEG which
// will get patched into this routine as the destination segment.
uint8_t JUMP_VECTOR[] = {
  0xEA, 0x00, 0x00, 0x00, 0x00
};

// // STOREALL 
// uint8_t JUMP_VECTOR[] = {
//   0xF1, 0x0F, 0x04, 0x00, 0x00
// };

#define JUMP_VECTOR_PATCH_OFFSET 3

// NMI vector. Not really a program, but using the program read function to read the vector
// address is conveneient.
uint8_t NMI_VECTOR[] = {
  0x00, 0x00, 0x00, 0x00
};
#define NMI_VECTOR_PATCH_OFFSET 2

// Storage to write to the stack during NMI
uint8_t NMI_STACK_BUFFER[] = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

// Register store routine.
// Six NOPs have been padded to the front of the STORE routine to hide it from appearing in
// client cycle traces.

// The store program for NMI-based program termination.
const uint8_t STORE_PROGRAM_NMI[] = {
  0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
  0xE7, 0xFE, 0x89, 0xD8, 0xE7, 0xFE, 0x89, 0xC8, 0xE7, 0xFE, 0x89, 0xD0, 0xE7, 0xFE, 0x58, 0xE7,
  0xFE, 0x58, 0xE7, 0xFE, 0x58, 0xE7, 0xFE, 0x8C, 0xD0, 0xE7, 0xFE, 0x89, 0xE0, 0xE7, 0xFE, 0x8C,
  0xD8, 0xE7, 0xFE, 0x8C, 0xC0, 0xE7, 0xFE, 0x89, 0xE8, 0xE7, 0xFE, 0x89, 0xF0, 0xE7, 0xFE, 0x89,
  0xF8, 0xE7, 0xFE, 0xB0, 0xFF, 0xE6, 0xFD
};

// The STORE program for inline program termination. This requires queue state availability.
const uint8_t STORE_PROGRAM_INLINE[] = {
  0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
  0xE7, 0xFE, 0x89, 0xD8, 0xE7, 0xFE, 0x89, 0xC8, 0xE7, 0xFE, 0x89, 0xD0, 0xE7, 0xFE, 0x8C, 0xD0,
  0xE7, 0xFE, 0x89, 0xE0, 0xE7, 0xFE, 0xB8, 0x00, 0x00, 0x8E, 0xD0, 0xB8, 0x04, 0x00, 0x89, 0xC4,
  0x9C, 0xE8, 0x00, 0x00, 0x8C, 0xC8, 0xE7, 0xFE, 0x8C, 0xD8, 0xE7, 0xFE, 0x8C, 0xC0, 0xE7, 0xFE,
  0x89, 0xE8, 0xE7, 0xFE, 0x89, 0xF0, 0xE7, 0xFE, 0x89, 0xF8, 0xE7, 0xFE, 0xB0, 0xFF, 0xE6, 0xFD
};

const uint8_t NEC_PREFETCH_PROGRAM[] = {
  0x63, 0xC0
};

uint8_t COMMAND_BUFFER[MAX_COMMAND_BYTES] = { 0 };

uint32_t CYCLE_NUM = 0;

command_func V_TABLE[] = {
  &cmd_version,
  &cmd_reset,
  &cmd_load,
  &cmd_cycle,
  &cmd_read_address_latch,
  &cmd_read_status,
  &cmd_read_8288_command,
  &cmd_read_8288_control,
  &cmd_read_data_bus,
  &cmd_write_data_bus,
  &cmd_finalize,
  &cmd_begin_store,
  &cmd_store,
  &cmd_queue_len,
  &cmd_queue_bytes,
  &cmd_write_pin,
  &cmd_read_pin,
  &cmd_get_program_state,
  &cmd_get_last_error,
  &cmd_get_cycle_state,
  &cmd_null,
  &cmd_prefetch_store,
  &cmd_read_address,
  &cmd_cpu_type,
  &cmd_emu8080,
  &cmd_prefetch,
  &cmd_init_screen,
  &cmd_invalid
};

const size_t MAX_CMD = (sizeof V_TABLE / sizeof(command_func));

char LAST_ERR[MAX_ERR_LEN] = { 0 };

const char *CPU_TYPE_STRINGS[] = {
  "Undetected",
  "i8088",
  "i8086",
  "NEC V20",
  "NEC V30",
  "i80188",
  "i80186",
  "i80286"
};
const char CPU_TYPE_COUNT = sizeof(CPU_TYPE_STRINGS) / sizeof(CPU_TYPE_STRINGS[0]);

const char *FPU_TYPE_STRINGS[] = {
  "None",
  "i8087",
};

// Strings for printing bus transfer cycles.
const char *CYCLE_STRINGS[] = {
  "T1", "T2", "T3", "T4", "Tw", "Ti"
};

const char *SEGMENT_STRINGS[] = {
  "ES", "SS", "CS", "DS"
};

// Specialize BoardType for the chosen HatType
using BoardType = BoardTypeBase<HatType>;

// Instantiate board and controller
BoardType Board;
BoardController<BoardType, HatType> Controller(Board); // Uses default Hat constructor
// Or if you want to pass Hat constructor parameters:
// BoardController<BoardType, HatType> Controller(Board, true); // Pass emulate_bus_controller = true

bool screen_initialized = false;
bool screen_init_requested = false;

//GigaDisplay_GFX display;
//GigaDisplay screen_impl(display);

// Main Sketch setup routine
void setup() {
  // Open the USB CDC serial port for in-band communication.
  // We run the client-server protocol over this port.
  INBAND_SERIAL.begin(BAUD_RATE);
  // Wait for the serial port to be ready.
  while (!INBAND_SERIAL)
    ;

  // Board initialization sets up the debug serial port.
  Board.init();

  // Wait for things to settle down.
  delayMicroseconds(200);

  Board.debugPrintln(DebugType::SETUP, "In setup()...");
  

//  // Set all output pins to OUTPUT
//  for (size_t p = 0; p < (sizeof OUTPUT_PINS / sizeof OUTPUT_PINS[0]); p++) {
//    pinMode(OUTPUT_PINS[p], OUTPUT);
//  }
//  // Set all input pins to INPUT
//  for (size_t p = 0; p < (sizeof INPUT_PINS / sizeof INPUT_PINS[0]); p++) {
//#ifdef ARDUINO_GIGA
//    //pinMode(INPUT_PINS[p], INPUT_PULLUP);
//    pinMode(INPUT_PINS[p], INPUT);
//#else
//    pinMode(INPUT_PINS[p], INPUT);
//#endif
//  }

#ifdef ARDUINO_GIGA
  Board.debugPrintln(DebugType::SETUP, "Running on Arduino Giga...");
#endif

  //i8288_status();

#if HAT_8087_V1
  debugPrintlnColor(ansi::bright_cyan, "8087 Hat specified!");
#endif

  // Patch the jumps in programs that jump
  patch_vector_pgm(JUMP_VECTOR, LOAD_SEG, JUMP_VECTOR_PATCH_OFFSET);
  patch_vector_pgm(SETUP_PROGRAM, LOAD_SEG, SETUP_PROGRAM_PATCH_OFFSET);
  patch_vector_pgm(NMI_VECTOR, STORE_SEG, NMI_VECTOR_PATCH_OFFSET);

  debugPrintlnColor(ansi::bright_cyan, "Identifying CPU...");
  cpu_id();

  #ifdef ARDUINO_GIGA
    pinMode(86, OUTPUT);
    pinMode(87, OUTPUT);  
    pinMode(88, OUTPUT);
    // Turn LED green
    digitalWrite(88, HIGH);
  #endif
  DEBUG_SERIAL.flush();
  clear_error();

  #if defined(ARDUINO_GIGA)
    #if GIGA_DISPLAY_SHIELD
      static GigaDisplay_GFX display;
      static GigaDisplay screen_impl(display);
      screen = &screen_impl;
      Board.debugPrintln(DebugType::SETUP, "Using Giga Display Shield!...");
    #else
      // Optional stub class for headless builds
      class NullDisplay : public Display {};
      static NullDisplay nullDisplay;
      screen = &nullDisplay;
    #endif
  #endif

  SERVER.c_state = WaitingForCommand;
  CPU.v_state = Reset;

  //screen->init();
  //beep(100);
  Board.debugPrintln(DebugType::SETUP, "Arduino8088 Server Initialized! Waiting for commands...");
}

void reset_cpu_struct(bool reset_load_regs) {

  // Retain detected cpu type, width and emulation flags.
  CpuType cpu_type = CPU.cpu_type;
  cpu_width_t width = CPU.width;
  bool do_emulation = CPU.do_emulation;
  
  // Make a copy of the register values if needed
  uint8_t load_regs_backup[sizeof(registers1_t)];
  if (!reset_load_regs) {
    memcpy(load_regs_backup, (const void*)&CPU.load_regs, sizeof(registers1_t));
  }

  // Zero the CPU struct
  memset(&CPU, 0, sizeof CPU);

  if (!reset_load_regs) {
    // Restore regs
    memcpy((void*)&CPU.load_regs, (const void*)load_regs_backup, sizeof(registers1_t));
  }

  // Restore retained values
  CPU.cpu_type = cpu_type;
  CPU.width = width;
  CPU.do_emulation = do_emulation;

  CPU.state_begin_time = 0;
  change_state(Reset);
  CPU.data_bus = 0x00;
  
}

bool cpu_id() {

  Board.debugPrintln(DebugType::ID, "cpu_id(): resetting CPU...");
  CpuResetResult reset_result = Controller.resetCpu();
  if (!reset_result.success) {
    Board.debugPrintln(DebugType::ID, "cpu_id(): Failed to reset CPU!");
    set_error("Failed to reset CPU!");
    return false;
  }
  else {
    Board.debugPrintln(DebugType::ID, "cpu_id(): CPU reset successful.");
  }

#if defined(CPU_186)
  // We can detect 188 vs 186 here. No need to enter CPU id program as we don't support
  // any other variants with this pinout.
  if (CPU.width == BusWidthEight) {
    CPU.cpu_type = CpuType::i80188;
  } else {
    CPU.cpu_type = CpuType::i80186;
  }
  Board.debugPrint(DebugType::ID, "cpu_id(): Detected CPU: ");
  Board.debugPrintln(DebugType::ID, CPU_TYPE_STRINGS[static_cast<size_t>(CPU.cpu_type)]);
  return true;
#elif defined(CPU_286)
  CPU.cpu_type = CpuType::i80286;
  Board.debugPrint(DebugType::ID, "cpu_id(): Detected CPU: ");
  Board.debugPrintln(DebugType::ID, CPU_TYPE_STRINGS[static_cast<size_t>(CPU.cpu_type)]);
  return true;
#endif
  

  change_state(CpuId);
  uint32_t timeout = 0;
  while (CPU.v_state != Load) {
    cycle();
    timeout++;
    if (timeout > 200) {
      Board.debugPrintln(DebugType::ID, "cpu_id(): CPU ID timeout!");
      set_error("CPU ID timeout!");
      return false;
    }
  }

  size_t t_idx = static_cast<size_t>(CPU.cpu_type);
  if (t_idx < CPU_TYPE_COUNT) {
    Board.debugPrint(DebugType::ID, "cpu_id(): Detected CPU: ");
    Board.debugPrintln(DebugType::ID, CPU_TYPE_STRINGS[t_idx]);
  } else {
    Board.debugPrintln(DebugType::ID, "Bad CPU type!");
    return false;
  }

  return true;
}

// Read a byte from the data bus. The half of the bus to read is determined
// by BHE and A0.
uint8_t data_bus_read_byte() {
  CPU.data_bus = data_bus_read(CPU.data_width);
  if (!READ_BHE_PIN) {
    // High byte is active.
    return (uint8_t)(CPU.data_bus >> 8);
  } else {
    // Low byte is active.
    return (uint8_t)CPU.data_bus;
  }
}

void data_bus_set_byte(uint8_t byte) {
  if (!READ_BHE_PIN) {
    // High byte is active.
    CPU.data_bus = ((uint16_t)byte) << 8;
  } else {
    // Low byte is active.
    CPU.data_bus = (uint16_t)byte;
  }
}

void clear_error() {
  strncpy(LAST_ERR, "No error", MAX_ERR_LEN - 1);
}

void set_error(const char *msg) {
  strncpy(LAST_ERR, msg, MAX_ERR_LEN - 1);
  DEBUG_SERIAL.println("");
  debugPrintlnColor(ansi::red, "************ ERROR ************");
  debugPrintlnColor(ansi::red, LAST_ERR);
  debugPrintlnColor(ansi::red, "*******************************");
  error_beep();
}

// Send a failure code byte in response to a failed command
void send_fail() {
  INBAND_SERIAL.write((uint8_t)RESPONSE_FAIL);
}

// Send the success code byte in response to a successful command
void send_ok() {
  INBAND_SERIAL.write((uint8_t)RESPONSE_OK);
}

void debug_proto(const char *msg) {
#if DEBUG_PROTO
  DEBUG_SERIAL.print("## ");
  DEBUG_SERIAL.print(msg);
  DEBUG_SERIAL.println(" ##");
#endif
}

void debug_cmd(const char *cmd, const char *msg) {
#if DEBUG_PROTO
  DEBUG_SERIAL.print("## cmd ");
  DEBUG_SERIAL.print(cmd);
  DEBUG_SERIAL.print(": ");
  DEBUG_SERIAL.print(msg);
  DEBUG_SERIAL.println(" ##");
#endif
}

// Server command - Version
// Send server identifier 'ard8088' followed by protocol version number in binary
bool cmd_version() {
  debug_cmd("VERSION", "In cmd");

  const char msg[] = "ardx86 ";
  INBAND_SERIAL.write((const uint8_t *)msg, sizeof(msg) - 1);
  //INBAND_SERIAL.write((uint8_t *)VERSION_DAT, VERSION_DAT_LEN);
  INBAND_SERIAL.write(VERSION_NUM);
  FLUSH;
  delay(10);  // let USB complete the transaction
  
  Controller.getBoard().debugPrintln(DebugType::CMD, "Got version query!");
  return true;
}

// Server command - Reset
// Attempt to reset the CPU and report status.
// This will be rarely used by itself as the register state is not set up. The Load
// command will reset the CPU and set register state.
bool cmd_reset() {
  debug_cmd("RESET", "In cmd_reset()");
  CpuResetResult result;
  snprintf(LAST_ERR, MAX_ERR_LEN, "NO ERROR");

  result = Controller.resetCpu();
  if (result.success) {
    change_state(Execute);
  }
  return result.success;
}

// Server command - Cpu type
// Return the detected CPU type and the queue status availability bit in MSB
bool cmd_cpu_type() {
  debug_cmd("CPU_TYPE", "In cmd_cpu_type()");
  snprintf(LAST_ERR, MAX_ERR_LEN, "NO ERROR");

  uint8_t byte = (uint8_t)CPU.cpu_type;
  // Set queue status available bit
  if (CPU.have_queue_status) {
    byte |= 0x80;
  }

  // Set FPU present bit
  if (CPU.fpu_type != 0) {
    byte |= 0x40;
  }

  INBAND_SERIAL.write(byte);
  return true;
}

// Server command - Cycle
// Execute the specified number of CPU cycles.
bool cmd_cycle() {
  uint8_t cycle_ct = COMMAND_BUFFER[0];
  for (uint8_t i = 0; i < cycle_ct; i++) {
    cycle();
  }
  return true;
}

// Server command - Load
// Load the specified register state into the CPU.
// 
// This command takes one byte first, which indicates the type of registers to load.
// On 8088-80186, this command takes 28 bytes, which correspond to the word values of each of the 14
// CPU registers.
// On 80286, this command takes 102 bytes, which correspond to the LOADALL structure.

bool cmd_load() {

  //DEBUG_SERIAL.println(">> Got load!");
  snprintf(LAST_ERR, MAX_ERR_LEN, "NO ERROR");
  volatile uint8_t *read_p = nullptr;
  uint8_t reg_type = COMMAND_BUFFER[0];
  bool read_result = false;

  switch (reg_type) {
    case 0:
      Controller.getBoard().debugPrintln(DebugType::LOAD, "## cmd_load(): Reading register struct type: 8088-80186");
      // 8088-80186 register load
      // This is the default register load type.
      read_result = readParameterBytes(COMMAND_BUFFER, sizeof(COMMAND_BUFFER), sizeof(registers1_t));

      if (!read_result) {
        Controller.getBoard().debugPrintln(DebugType::ERROR, "## cmd_load(): Timed out reading parameter bytes");
        set_error("Failed to read parameter bytes");
        return false;
      }

      // Write raw command bytes over register struct.
      // All possible bit representations are valid.
      read_p = reinterpret_cast<volatile uint8_t*>(&CPU.load_regs);
      for (size_t i = 0; i < sizeof(registers1_t); i++) {
        *read_p++ = COMMAND_BUFFER[i];
      }

      patch_load_pgm(LOAD_PROGRAM, &CPU.load_regs);
      patch_brkem_pgm(EMU_ENTER_PROGRAM, &CPU.load_regs);

      CPU.load_regs.flags &= CPU_FLAG_DEFAULT_CLEAR;
      CPU.load_regs.flags |= CPU_FLAG_DEFAULT_SET_8086;
    break;

    case 1:
      Controller.getBoard().debugPrintln(DebugType::LOAD, "## cmd_load(): Reading register struct type: 80286 (LOADALL)");
      read_result = readParameterBytes(COMMAND_BUFFER, sizeof(COMMAND_BUFFER), sizeof(Loadall286));
      if (!read_result) {
        Controller.getBoard().debugPrintln(DebugType::ERROR, "## cmd_load(): Timed out reading parameter bytes");
        set_error("Failed to read parameter bytes");
        return false;
      }

      // Write raw command bytes over register struct.
      read_p = reinterpret_cast<volatile uint8_t*>(&CPU.loadall_regs);
      for (size_t i = 0; i < sizeof(Loadall286); i++) {
        *read_p++ = COMMAND_BUFFER[i];
      }

      CPU.loadall_regs.FLAGS &= CPU_FLAG_DEFAULT_CLEAR;
      CPU.loadall_regs.FLAGS |= CPU_FLAG_DEFAULT_SET_286;
    break;

    default:
      set_error("Invalid register type");
      return false;
  }

  CpuResetResult result = Controller.resetCpu();
  if (!result.success) {
    //set_error("Failed to reset CPU");
    Controller.getBoard().debugPrintln(DebugType::ERROR, "Failed to reset CPU!");
    return false;
  }
  change_state(JumpVector);

  // Run CPU and wait for load to finish
  int load_timeout = 0;
  while (CPU.v_state != Execute) {
    cycle();
    load_timeout++;

    if (load_timeout > LOAD_TIMEOUT) {
      // Something went wrong in load program
      set_error("Load timeout");
      return false;
    }
  }

#if LOAD_INDICATOR
  DEBUG_SERIAL.print(".");
#endif
  debug_proto("LOAD DONE");
  return true;
}

// Server command - ReadAddressLatch
// Read back the contents of the address latch as a sequence of 3 bytes (little-endian)
bool cmd_read_address_latch() {
  INBAND_SERIAL.write((uint8_t)(CPU.address_latch & 0xFF));
  INBAND_SERIAL.write((uint8_t)((CPU.address_latch >> 8) & 0xFF));
  INBAND_SERIAL.write((uint8_t)((CPU.address_latch >> 16) & 0xFF));
  return true;
}

// Server command - ReadAddress
// Read back the contents of the address bus as a sequence of 3 bytes (little-endian)
bool cmd_read_address() {
  //read_address_pins(true);
  CPU.address_bus = Controller.readAddressBus(true);
  INBAND_SERIAL.write((uint8_t)(CPU.address_bus & 0xFF));
  INBAND_SERIAL.write((uint8_t)((CPU.address_bus >> 8) & 0xFF));
  INBAND_SERIAL.write((uint8_t)((CPU.address_bus >> 16) & 0xFF));
  return true;
}

bool cmd_invalid() {
  DEBUG_SERIAL.println("Called cmd_invalid!");
  return false;
}

// Server command - ReadStatus
// Return the value of the CPU status lines S0-S5 and QS0-QS1
bool cmd_read_status() {
  CPU.status0 = Controller.readCpuStatusLines();
  INBAND_SERIAL.write(CPU.status0);
  return true;
}

// Server command - Read8288Command
bool cmd_read_8288_command() {
  CPU.command_bits = Controller.readBusControllerCommandLines();
  //read_8288_command_bits();
  INBAND_SERIAL.write(CPU.command_bits);
  return true;
}

// Server command - Read8288Control
bool cmd_read_8288_control() {
  Controller.readBusControllerControlLines();
  //read_8288_control_bits();
  INBAND_SERIAL.write(CPU.control_bits);
  return true;
}

// Server command - ReadDataBus
bool cmd_read_data_bus() {
  INBAND_SERIAL.write((uint8_t)CPU.data_bus);
  INBAND_SERIAL.write((uint8_t)(CPU.data_bus >> 8));
  return true;
}

// Server command - WriteDataBus
// Takes an argument of 2 bytes.
// Sets the data bus to the provided value. On 8-bit CPUs the upper byte is ignored.
// This should not be called for CODE fetches after we have called cmd_prefetch_store(),
// unless a flow control operation occurs that flushes the queue and returns us to
// within original program boundaries.
bool cmd_write_data_bus() {
  if (CPU.bus_state_latched == CODE) {
    // We've just been instructed to write a normal fetch byte to the bus.
    // If we were prefetching the store program, reset this status as a queue
    // flush must have executed (or we goofed up...)
    CPU.prefetching_store = false;
    CPU.s_pc = 0;
  }

  CPU.data_bus = (uint16_t)COMMAND_BUFFER[0];
  CPU.data_bus |= ((uint16_t)COMMAND_BUFFER[1] << 8);
  CPU.data_type = QueueDataType::Program;

  Controller.getBoard().debugPrint(DebugType::CMD, "## cmd_write_data_bus(): Writing data bus: ");
  Controller.getBoard().debugPrintln(DebugType::CMD, CPU.data_bus, HEX);

  return true;
}

// Server command - PrefetchStore
// Instructs the CPU server to load the next byte of the Store (or EmuExit) program early
// Should be called in place of cmd_write_data_bus() by host on T3/TwLast when
// program bytes have been exhausted.
// (When we are prefetching past execution boundaries during main program execution)
bool cmd_prefetch_store() {

  if (CPU.in_emulation) {
    // Prefetch the EmuExit program
    if (CPU.s_pc >= sizeof EMU_EXIT_PROGRAM) {
      set_error("EmuExit program underflow");
      return false;
    }

#if DEBUG_STORE
    debugPrintColor(ansi::yellow, "## PREFETCH_EMU_EXIT: s_pc: ");
#endif

    CPU.prefetching_store = true;
    CPU.data_bus = read_program(EMU_EXIT_PROGRAM, sizeof EMU_EXIT_PROGRAM, &CPU.s_pc, CPU.address_latch, CPU.data_width);
    CPU.data_type = QueueDataType::ProgramEnd;
  } else {
    // Prefetch the Store program
    if (CPU.s_pc >= sizeof STORE_PROGRAM_INLINE) {
      set_error("## Store program underflow!");
      return false;
    }

#if DEBUG_STORE
    debugPrintColor(ansi::yellow, "## PREFETCH_STORE: s_pc: ");
#endif

    CPU.prefetching_store = true;
    CPU.data_bus = read_program(STORE_PROGRAM_INLINE, sizeof STORE_PROGRAM_INLINE, &CPU.s_pc, CPU.address_latch, CPU.data_width);
    CPU.data_type = QueueDataType::ProgramEnd;
  }

#if DEBUG_STORE
  debugPrintColor(ansi::yellow, CPU.s_pc);
  debugPrintColor(ansi::yellow, " addr: ");
  debugPrintColor(ansi::yellow, CPU.address_latch, 16);
  debugPrintColor(ansi::yellow, " data: ");
  debugPrintlnColor(ansi::yellow, CPU.data_bus, 16);
#endif

  return true;
}

// Server command - Finalize
// Sets the data bus flag to DATA_PROGRAM_END, so that the Execute state can terminate
// on the next instruction queue fetch
bool cmd_finalize() {
  if (CPU.v_state == Execute) {
    change_state(ExecuteFinalize);

    // Wait for execute done state
    int execute_ct = 0;
    int timeout = FINALIZE_TIMEOUT;
    if (CPU.in_emulation) {
      // We need more time to exit emulation mode
      timeout = FINALIZE_EMU_TIMEOUT;
    }
    while (CPU.v_state != ExecuteDone) {
      cycle();
      execute_ct++;



      if (execute_ct > timeout) {
        set_error("cmd_finalize(): state timeout");
        return false;
      }
    }
    return true;
  } else {
    error_beep();
    set_error("cmd_finalize(): wrong state: ");
    DEBUG_SERIAL.println(CPU.v_state);
    return false;
  }
}

// Server command - BeginStore
// Execute state must be in ExecuteDone before intiating BeginStore command
//
bool cmd_begin_store(void) {
  /*
  char err_msg[30];

  // Command only valid in ExecuteDone state
  if(CPU.v_state != ExecuteDone) {
    snprintf(err_msg, 30, "BeginStore: Wrong state: %d ", CPU.v_state);
    set_error(err_msg);
    return false;
  }

  change_state(Store);
  */
  return true;
}

// Server command - Store
//
// Returns values of registers in the following order, little-endian
// AX, BX, CX, DX, SS, SP, FLAGS, IP, CS, DS, ES, BP, SI, DI
// Execute state must be in StoreDone before executing Store command
bool cmd_store(void) {

#if DEBUG_STORE
  debugPrintColor(ansi::bright_magenta, "## In STORE: s_pc is: ");
  debugPrintlnColor(ansi::bright_magenta, CPU.s_pc);
#endif

  char err_msg[30];
  // Command only valid in ExecuteDone state
  if (CPU.v_state != ExecuteDone) {
    snprintf(err_msg, 30, "## STORE: Wrong state: %d ", CPU.v_state);
    set_error(err_msg);
    return false;
  }

  change_state(Store);

  int store_timeout = 0;

  // Cycle CPU until Store complete
  while (CPU.v_state != StoreDone) {
    cycle();
    store_timeout++;

    if (store_timeout > 500) {
      debugPrintlnColor(ansi::bright_red, "## STORE: Timeout! ##");
      snprintf(err_msg, 30, "StoreDone timeout.");
      error_beep();
      return false;
    }
  }

#if DEBUG_STORE
  debugPrintColor(ansi::bright_magenta, "## STORE: Flags are: ");
  debugPrintlnColor(ansi::bright_magenta, CPU.post_regs.flags, 16);

  if (!CPU.nmi_terminate) {
    // We didn't use NMI to terminate code. which means we used the inline STORE routine,
    // which means we need to convert the registers struct.
    debugPrintlnColor(ansi::bright_magenta, "## STORE: Converting registers from STORE INLINE format...");
    convert_inline_registers(&CPU.post_regs);
  }
#endif

  // Dump final register state to Serial port
  uint8_t *reg_p = (uint8_t *)&CPU.post_regs;
  for (size_t i = 0; i < sizeof CPU.post_regs; i++) {
    INBAND_SERIAL.write(reg_p[i]);
  }

#if STORE_INDICATOR
  DEBUG_SERIAL.print("?");
#endif
  change_state(Done);
  return true;
}


// Server command - QueueLen
// Return the length of the instruction queue in bytes
bool cmd_queue_len(void) {
  INBAND_SERIAL.write(static_cast<uint8_t>(CPU.queue.len()));
  return true;
}

// Server command - QueueBytes
// Return the contents of the instruction queue, from 0-6 bytes.
bool cmd_queue_bytes(void) {
  for (size_t i = 0; i < CPU.queue.len(); i++) {
    INBAND_SERIAL.write(static_cast<uint8_t>(CPU.queue.read_byte(i)));
  }
  return true;
}

// Server command - Write pin
// Sets the value of the specified CPU input pin
bool cmd_write_pin(void) {

  uint8_t pin_idx = COMMAND_BUFFER[0];
  uint8_t pin_val = COMMAND_BUFFER[1] & 0x01;

  if (pin_idx < sizeof WRITE_PINS) {
    //uint8_t pin_no = WRITE_PINS[pin_idx];

    switch (pin_idx) {
      case 0: // READY pin
#if DEBUG_PIN_CMD
        debugPrintColor(ansi::cyan, "Setting READY pin to: ");
        debugPrintlnColor(ansi::cyan, pin_val);
#endif
        Controller.writePin(OutputPin::Ready, pin_val);
        break;

      case 1: // TEST pin
#if DEBUG_PIN_CMD
        debugPrintColor(ansi::cyan, "Setting TEST pin to: ");
        debugPrintlnColor(ansi::cyan, pin_val);
#endif
        Controller.writePin(OutputPin::Test, pin_val);
        break;

      case 2: // INTR pin
#if DEBUG_PIN_CMD
        debugPrintColor(ansi::cyan, "Setting INTR pin to: ");
        debugPrintlnColor(ansi::cyan, pin_val);
#endif
        Controller.writePin(OutputPin::Intr, pin_val);
        break;

      case 3: // NMI pin
#if DEBUG_PIN_CMD
        debugPrintColor(ansi::cyan, "Setting NMI pin to: ");
        debugPrintlnColor(ansi::cyan, pin_val);
#endif
        Controller.writePin(OutputPin::Nmi, pin_val);
        break;

      default:
        error_beep();
        return false;
    }
    return true;
  } else {
    // Invalid pin
    error_beep();
    return false;
  }
}

// Server command - Read pin
bool cmd_read_pin(void) {
  // Not implemented
  INBAND_SERIAL.write((uint8_t)0);
  return true;
}

// Server command - Get program state
bool cmd_get_program_state(void) {
  INBAND_SERIAL.write((uint8_t)CPU.v_state);
  return true;
}

// Server command - Get last error
bool cmd_get_last_error(void) {
  INBAND_SERIAL.write(LAST_ERR);
  return true;
}

// Server command - Get Cycle State
// Returns all the state info typically needed for a single cycle
// Parameter: One byte, flags. Bit 0 set to 1 will cycle CPU first before 
//            returning the state.
// Returns 11 bytes
bool cmd_get_cycle_state(void) {
  bool do_cycle = (bool)(COMMAND_BUFFER[0] & 0x01);
  if (do_cycle) {
    // Perform a cycle if requested
    cycle();
  }

  //CPU.status0 = Controller.readCpuStatusLines();
  CPU.command_bits = Controller.readBusControllerCommandLines();
  CPU.control_bits = Controller.readBusControllerControlLines();

  //read_8288_command_bits();
  //read_8288_control_bits();
  uint8_t server_state = ((uint8_t)CPU.v_state) & 0x3F;
  uint8_t cpu_state_byte = 0;

  cpu_state_byte |= ((uint8_t)(CPU.last_bus_cycle)) & 0x07; // Bits 0-2: Bus cycle

  INBAND_SERIAL.write(server_state); // Byte 0
  INBAND_SERIAL.write(cpu_state_byte); // Byte 1
  INBAND_SERIAL.write(CPU.status0); // Byte 2
  INBAND_SERIAL.write(CPU.control_bits); // Byte 3
  INBAND_SERIAL.write(CPU.command_bits); // Byte 4

  INBAND_SERIAL.write((uint8_t)(CPU.address_bus & 0xFF)); // Bytes 5-8
  INBAND_SERIAL.write((uint8_t)((CPU.address_bus >> 8 ) & 0xFF));
  INBAND_SERIAL.write((uint8_t)((CPU.address_bus >> 16) & 0xFF));
  INBAND_SERIAL.write((uint8_t)((CPU.address_bus >> 24) & 0xFF));

  INBAND_SERIAL.write((uint8_t)(CPU.data_bus & 0xFF)); // Bytes 9-10
  INBAND_SERIAL.write((uint8_t)(CPU.data_bus >> 8));
  return true;
}

// Server command - Enter emulation mode
bool cmd_emu8080(void) {
  if ((CPU.cpu_type == CpuType::necV20) || (CPU.cpu_type == CpuType::necV30)) {
    // Simply toggle the emulation flag
    CPU.do_emulation = true;
#if DEBUG_EMU
    DEBUG_SERIAL.println("## cmd_emu8080(): Enabling 8080 emulation mode! ##");
#endif
    return true;
  }
// Unsupported CPU!
#if DEBUG_EMU
  DEBUG_SERIAL.println("## cmd_emu8080(): Bad CPU type ## ");
#endif
  return false;
}

// Server command - prefetch
bool cmd_prefetch(void) {
  if ((CPU.cpu_type == CpuType::necV20) || (CPU.cpu_type == CpuType::necV30)) {
    // Simply toggle the emulation flag
    CPU.do_prefetch = true;
#if DEBUG_EMU
    DEBUG_SERIAL.println("## cmd_prefetch(): Enabling VX0 prefetch ##");
#endif
    return true;
  }

// Unsupported CPU!
#if DEBUG_EMU
  DEBUG_SERIAL.println("## cmd_prefetch(): Bad CPU type ## ");
#endif
  return false;
}

// The Giga display shield takes a few seconds to initialize, so we don't 
// want to do it in setup. The client can request display initialization instead
// once it has established a connection.  The client should wait approximately 
// 3 seconds after sending the init command.
// Returns one data byte. A value of 0 indicates no display is present.
// A value of 1 indicates the display is present and will be initialized.
bool cmd_init_screen() {
  uint8_t byte0 = 0;
  #if GIGA_DISPLAY_SHIELD
    byte0 = 1;
    screen_init_requested = true;
  #endif
  INBAND_SERIAL.write(byte0);
  return true;
}

bool cmd_null() {
  return true;
}

void patch_vector_pgm(uint8_t *pgm, uint16_t seg, size_t offset) {
  *((uint16_t *)&pgm[offset]) = seg;
}

void patch_load_pgm(uint8_t *pgm, volatile registers1_t *reg) {
  *((uint16_t *)pgm) = reg->flags;
  *((uint16_t *)&pgm[LOAD_BX]) = reg->bx;
  *((uint16_t *)&pgm[LOAD_CX]) = reg->cx;
  *((uint16_t *)&pgm[LOAD_DX]) = reg->dx;
  *((uint16_t *)&pgm[LOAD_SS]) = reg->ss;
  *((uint16_t *)&pgm[LOAD_DS]) = reg->ds;
  *((uint16_t *)&pgm[LOAD_ES]) = reg->es;
  *((uint16_t *)&pgm[LOAD_SP]) = reg->sp;
  *((uint16_t *)&pgm[LOAD_BP]) = reg->bp;
  *((uint16_t *)&pgm[LOAD_SI]) = reg->si;
  *((uint16_t *)&pgm[LOAD_DI]) = reg->di;
  *((uint16_t *)&pgm[LOAD_AX]) = reg->ax;
  *((uint16_t *)&pgm[LOAD_IP]) = reg->ip;
  *((uint16_t *)&pgm[LOAD_CS]) = reg->cs;
}

void patch_brkem_pgm(uint8_t *pgm, volatile registers1_t *regs) {
#if DEBUG_EMU
  static char buf[20];
  DEBUG_SERIAL.println("## Patching BRKEM program ##");
  snprintf(buf, 20,
           "CS: %04X IP: %04X",
           regs->cs,
           regs->ip);
  DEBUG_SERIAL.println(buf);
#endif
  uint16_t *word_ptr = (uint16_t *)pgm;
  *word_ptr++ = regs->ip;
  *word_ptr = regs->cs;
}

// Fix up the old-style inline regs by swapping fields.
void convert_inline_registers(volatile void *inline_regs) {

  registers2_t *regs2 = (registers2_t *)inline_regs;

  uint16_t ip = regs2->ip;
  uint16_t cs = regs2->cs;
  uint16_t ss = regs2->ss;
  uint16_t sp = regs2->sp;

  registers1_t *regs1 = (registers1_t *)inline_regs;

  regs1->ip = ip;
  regs1->cs = cs;
  regs1->ss = ss;
  regs1->sp = sp;
}

void print_registers(registers1_t *regs) {
  static char buf[130];
  static char flag_buf[17];

  if (!regs) {
    return;
  }

  snprintf(buf, 130,
           "AX: %04x BX: %04x CX: %04x DX: %04x\n"
           "SP: %04x BP: %04x SI: %04x DI: %04x\n"
           "CS: %04x DS: %04x ES: %04x SS: %04x\n"
           "IP: %04x\n"
           "FLAGS: %04x",
           regs->ax, regs->bx, regs->cx, regs->dx,
           regs->sp, regs->bp, regs->si, regs->di,
           regs->cs, regs->ds, regs->es, regs->ss,
           regs->ip,
           regs->flags);

  DEBUG_SERIAL.println(buf);

  // Expand flag info
  uint16_t f = regs->flags;
  char c_chr = CPU_FLAG_CARRY & f ? 'C' : 'c';
  char p_chr = CPU_FLAG_PARITY & f ? 'P' : 'p';
  char a_chr = CPU_FLAG_AUX_CARRY & f ? 'A' : 'a';
  char z_chr = CPU_FLAG_ZERO & f ? 'Z' : 'z';
  char s_chr = CPU_FLAG_SIGN & f ? 'S' : 's';
  char t_chr = CPU_FLAG_TRAP & f ? 'T' : 't';
  char i_chr = CPU_FLAG_INT_ENABLE & f ? 'I' : 'i';
  char d_chr = CPU_FLAG_DIRECTION & f ? 'D' : 'd';
  char o_chr = CPU_FLAG_OVERFLOW & f ? 'O' : 'o';

  snprintf(
    flag_buf, 17,
    "1111%c%c%c%c%c%c0%c0%c1%c",
    o_chr, d_chr, i_chr, t_chr, s_chr, z_chr, a_chr, p_chr, c_chr);

  DEBUG_SERIAL.print("FLAGSINFO: ");
  DEBUG_SERIAL.println(flag_buf);
}

void print_cpu_state() {
  const size_t buf_len = 90;
  static char buf[buf_len];
  const size_t op_len = 9;  //(4 + 4 + 1)
  static char op_buf[op_len];
  static char q_buf[15];
  static char data_buf[6];
  size_t bus_str_width = 0;

  const char *ale_str = Controller.readALEPin() ? "A:" : "  ";

  bool mrdc = (CPU.command_bits & 0x01);
  bool amwc = (CPU.command_bits & 0x02);
  bool mwtc = (CPU.command_bits & 0x04);

  bool iorc = (CPU.command_bits & 0x08);
  bool aiowc = (CPU.command_bits & 0x10);
  bool iowc = (CPU.command_bits & 0x20);

  char rs_chr = !mrdc ? 'R' : '.';
  char aws_chr = !amwc ? 'A' : '.';
  char ws_chr = !mwtc ? 'W' : '.';

  char ior_chr = !iorc ? 'R' : '.';
  char aiow_chr = !aiowc ? 'A' : '.';
  char iow_chr = !iowc ? 'W' : '.';

  char reset_chr = READ_RESET_PIN ? 'R' : '.';
  char intr_chr = READ_INTR_PIN ? 'I' : '.';
  char inta_chr = '.';
  char nmi_chr = READ_NMI_PIN ? 'N' : '.';
  char bhe_chr = !READ_BHE_PIN ? 'B' : '.';

  char test_chr = READ_TEST_PIN ? 'T' : '.';
  char rq_chr = !READ_PIN_D03 ? 'R' : '.';
  char fint_chr = READ_PIN_D20 ? 'I' : '.';
  char lock_chr = !READ_LOCK_PIN ? 'L' : '.';

  char v_chr = MACHINE_STATE_CHARS[(size_t)CPU.v_state];
  uint8_t q = (CPU.status0 >> 6) & 0x03;
  char q_char = QUEUE_STATUS_CHARS[q];
  //char s = CPU.status0 & 0x07;
  //char rout_chr = '.';

  // Set the bus string width
  if (CPU.width == BusWidthEight) {
    bus_str_width = 2;
  } else {
    bus_str_width = 4;
  }

  // Get segment from S3 & S4 if supported by the current CPU.
  const char *seg_str = "  ";
  if(Controller.hasSegmentStatus()) {
    // Get segment from S3 & S4
    uint8_t seg = ((CPU.status0 & 0x18) >> 3) & 0x03;
    seg_str = SEGMENT_STRINGS[(size_t)seg];
  }

  // Make data bus string and set r/w indicators based on bus size
  data_buf[5] = { ' ' };
  const char *rd_str = "r";
  const char *wr_str = "w";
  if (CPU.data_width == ActiveBusWidth::EightLow) {
    // Write two hex digits, 0-padded
    snprintf(data_buf, 5, "%4.2X", (uint8_t)CPU.data_bus);
  } else if (CPU.data_width == ActiveBusWidth::EightHigh) {
    // Write two hex digits, 0-padded, left-aligned in 4-character field
    snprintf(data_buf, 5, "%-4.2X", (uint8_t)(CPU.data_bus >> 8));
  } else {
    rd_str = "R";
    wr_str = "W";
    // Write four hex digits, 0-padded
    snprintf(data_buf, 5, "%04X", CPU.data_bus);
  }

  // Make string for bus reads and writes
  op_buf[0] = 0;
  if ((!mrdc || !iorc) && CPU.bus_state == PASV) {
    snprintf(op_buf, op_len, "%s-> %s", rd_str, data_buf);
  } else if (!mwtc || !iowc) {
    snprintf(op_buf, op_len, "<-%s %s", wr_str, data_buf);
  } else {
    snprintf(op_buf, op_len, "%*s", (int)(4 + bus_str_width), "");
  }

  const char *q_str = CPU.queue.to_string();

  const char *t_str;
  if ((CPU.bus_cycle == T1) && (CPU.bus_state == PASV)) {
    // Convert T1 to Ti when passive bus
    t_str = "Ti";
  } else {
    t_str = Controller.getTCycleString(CPU.bus_cycle);
  }

// Set reset_out chr
#if CPU_186
  if (READ_PIN_D03) {
    rout_chr = 'r';
  }
#endif

  uint32_t address_bus = Controller.readAddressBus(true);
  int address_digits = Controller.getAddressDigits();

  snprintf(
      buf,
      buf_len,
      "%08ld %c %s[%0*lX][%0*lX]",
      CYCLE_NUM,
      v_chr,
      ale_str,
      address_digits, CPU.address_latch,
      address_digits, address_bus);

  DEBUG_SERIAL.print(buf);

  // Print the data bus if we don't have a multiplexed bus.
  // If the bus is multiplexed, we effectively print the data bus when printing the 
  // address bus.
  if (!Controller.hasMultiplexedBus()) {
    snprintf(
      buf,
      buf_len,
      "[%04X]",
      Controller.readDataBus(ActiveBusWidth::Sixteen, true));
  }

  DEBUG_SERIAL.print(buf);

  snprintf(
      buf,
      buf_len,    
      " %2s M:%c%c%c I:%c%c%c P:%c%c%c%c%c F:%c%c%c%c ",
      seg_str,
      rs_chr, aws_chr, ws_chr,
      ior_chr, aiow_chr, iow_chr,
      reset_chr, intr_chr, inta_chr, nmi_chr, bhe_chr,
      lock_chr, test_chr, rq_chr, fint_chr);

  DEBUG_SERIAL.print(buf);

#if defined(FPU_8087)
  snprintf(
      buf,
      buf_len,    
      "F:%c%c%c%c ",
      lock_chr, test_chr, rq_chr, fint_chr);
  DEBUG_SERIAL.print(buf);
#endif  

#if defined(CPU_286) 
  char ice0_chr = READ_ICE_PIN0 ? '1' : '0';
  char ice1_chr = READ_ICE_PIN1 ? '1' : '0';
  snprintf(
    buf, 
    buf_len,
    "C:%c%c ",
    ice0_chr, ice1_chr
  );
  DEBUG_SERIAL.print(buf);
#endif

  debugPrintColor(Controller.getBusStatusColor(CPU.bus_state), Controller.getBusStatusString(CPU.bus_state));

  snprintf(
    buf,
    buf_len,
    "[%0X] %s %8s | %c%d [%-*s]",
    CPU.status0,
    t_str,
    op_buf,
    q_char,
    CPU.queue.len(),
    CPU.queue.size() * 2,
    q_str);

  DEBUG_SERIAL.print(buf);

// Print queue status string if we have queue status pins available.
#if HAVE_QUEUE_STATUS
  if (q == QUEUE_FIRST) {
    // First byte of opcode read from queue. Decode it to opcode
    snprintf(q_buf, 15, " <-q %02X %s", CPU.qb, get_opcode_str(CPU.opcode, 0, false));
    DEBUG_SERIAL.print(q_buf);
  } else if (q == QUEUE_SUBSEQUENT) {
    if (!CPU.in_emulation && IS_GRP_OP(CPU.opcode) && CPU.q_fn == 1) {
      // Modrm was just fetched for a group opcode, so display the mnemonic now
      snprintf(q_buf, 15, " <-q %02X %s", CPU.qb, get_opcode_str(CPU.opcode, CPU.qb, true));
    } else {
      snprintf(q_buf, 15, " <-q %02X", CPU.qb);
    }
    DEBUG_SERIAL.print(q_buf);
  }
#endif

  DEBUG_SERIAL.println("");
}

void change_state(machine_state_t new_state) {
  switch (new_state) {
    case Reset:
      CPU.doing_reset = true;
      CPU.cpuid_counter = 0;
      CPU.cpuid_queue_reads = 0;
      CPU.v_pc = 0;
      CPU.s_pc = 0;
      break;
    case CpuSetup:
      CPU.v_pc = 0;
      break;
    case CpuId:
      CPU.doing_reset = false;
      CPU.doing_id = true;
      CPU.cpuid_counter = 0;
      CPU.cpuid_queue_reads = 0;
      CPU.v_pc = 0;
      break;
    case JumpVector:
      CPU.doing_reset = false;
      CPU.v_pc = 0;
      break;
    case Load:
      
      if (CPU.cpu_type == CpuType::i80286) {
        CPU.program = LOAD_PROGRAM_286;
        CPU.program_len = sizeof(LOAD_PROGRAM_286);
        CPU.v_pc = 0;
      }
      else {
        CPU.program = LOAD_PROGRAM;
        CPU.program_len = sizeof(LOAD_PROGRAM);
        // Set v_pc to 2 to skip flag bytes
        CPU.v_pc = 2;
      }

      CPU.program_pc = &CPU.v_pc;

      break;
    case LoadDone:
      break;
    case EmuEnter:
      CPU.stack_r_op_ct = 0;
      CPU.stack_w_op_ct = 0;
      // Set v_pc to 4 to skip IVT segment:offset
      CPU.v_pc = 4;
      break;
    case Execute:
      CPU.nmi_checkpoint = 0;
      CPU.v_pc = 0;
      CPU.s_pc = 0;
      if (CPU.do_emulation) {
        // Set v_pc to 4 to skip IVT segment:offset
        CPU.s_pc = 4;
      }
      break;
    case ExecuteFinalize:
      CPU.nmi_checkpoint = 0;
      CPU.nmi_buf_cursor = 0;  // Reset cursor for NMI stack buffer storage
      CPU.v_pc = 0;            // Reset PC for NMI vector "program"

      if (CPU.in_emulation) {
        CPU.program = EMU_EXIT_PROGRAM;
        CPU.program_len = sizeof(EMU_EXIT_PROGRAM);
        CPU.program_pc = &CPU.v_pc;
      } else if (CPU.nmi_terminate) {
        CPU.program = STORE_PROGRAM_NMI;
        CPU.program_len = sizeof(STORE_PROGRAM_NMI);
        CPU.program_pc = &CPU.s_pc;
      } else {
        CPU.program = STORE_PROGRAM_INLINE;
        CPU.program_len = sizeof(STORE_PROGRAM_INLINE);
        CPU.program_pc = &CPU.s_pc;
      }

      break;
    case ExecuteDone:
      break;
    case EmuExit:
      CPU.stack_r_op_ct = 0;
      CPU.stack_w_op_ct = 0;
      CPU.v_pc = 0;
      break;
    case Store:
      reverse_stack_buf();
      CPU.nmi_buf_cursor = 0;  // Reset cursor for NMI stack buffer storage
      // Take a raw uint8_t pointer to the register struct. Both x86 and Arduino are little-endian,
      // so we can write raw incoming data over the struct. Faster than logic required to set
      // specific members.
      CPU.readback_p = (uint8_t *)&CPU.post_regs;
      break;
    case StoreDone:
      break;
    case Done:
      break;
    default:
      // Unhandled state.
      break;
  }

  uint32_t state_end_time = micros();

  // Report time we spent in the previous state.
  if (CPU.state_begin_time != 0) {
    uint32_t elapsed = state_end_time - CPU.state_begin_time;
    Controller.getBoard().debugPrint(DebugType::STATE, "## Changing to state: ");
    Controller.getBoard().debugPrint(DebugType::STATE, MACHINE_STATE_STRINGS[(size_t)new_state]);
    Controller.getBoard().debugPrint(DebugType::STATE, ". Spent (");
    Controller.getBoard().debugPrint(DebugType::STATE, elapsed);
    Controller.getBoard().debugPrintln(DebugType::STATE, " us in previous state. ##");
  }

  CPU.state_begin_time = micros();
  CPU.v_state = new_state;
}

// Emulate a code fetch from the specified program.
uint16_t read_program(const uint8_t *program, size_t len, uint16_t *pc, uint32_t address, ActiveBusWidth width) {

  uint16_t data = 0x9090;

  if (*pc >= len) {
    return data;
  }

  if (width == ActiveBusWidth::EightLow) {
    data = program[(*pc)++];
  } else if (width == ActiveBusWidth::EightHigh) {
    // TODO: bounds checks
    //DEBUG_SERIAL.println("## Odd read ##");
    if ((*pc) > 0) {
      // This byte doesn't really matter, but we can simulate fetching more realistically by including it.
      // If this happens to be the start of the program though, it will just have to be 0.
      data = program[(*pc) - 1];
    }
    if (*pc < len) {
      data &= 0x00FF;
      data |= ((uint16_t)program[(*pc)++]) << 8;
    }
  } else {
    // 16-bit read.

    if ((address & 1) == 0) {
      // Even address
      //DEBUG_SERIAL.println("## Even read ##");
      data = program[(*pc)++];
      if (*pc < len) {
        data &= 0x00FF;
        data |= ((uint16_t)program[(*pc)++]) << 8;
      }
    } else {
      // Odd address.
      debugPrintlnColor(ansi::bright_red, "## read_program(): Odd 16-bit read, shouldn't happen! ##");
    }
  }

  return data;
}

void write_buffer(uint8_t *buffer, uint16_t *cursor, uint16_t data, uint32_t address, ActiveBusWidth width) {
  if (width == ActiveBusWidth::EightLow) {
    buffer[(*cursor)++] = (uint8_t)data;
  } else if (width == ActiveBusWidth::EightHigh) {
    buffer[(*cursor)++] = (uint8_t)(data >> 8);
  } else {
    // 16-bit read.
    if ((address & 1) == 0) {
      // Even address
      buffer[(*cursor)++] = (uint8_t)data;
      buffer[(*cursor)++] = (uint8_t)(data >> 8);
    } else {
      // Odd address.
      debugPrintlnColor(ansi::bright_red, "## write_buffer(): Odd 16-bit read, shouldn't happen! ##");
    }
  }
}

// Simulate fetching NOPs based on bus width.
uint16_t read_nops(ActiveBusWidth width) {
  if (width == ActiveBusWidth::EightLow) {
    return 0x90;
  } else {
    return 0x9090;
  }
}

void set_data_bus_width() {
  if (!READ_BHE_PIN) {
    if ((CPU.address_latch & 1) == 0) {
// BHE is active, and address is even. Bus width is 16.
#if DEBUG_BUS
      debugPrintlnColor(ansi::bright_yellow, "Bus width 16");
#endif
      CPU.data_width = ActiveBusWidth::Sixteen;
    } else {
// BHE is active, and address is odd. Bus width is EightHigh.
#if DEBUG_BUS
      debugPrintlnColor(ansi::bright_yellow, "Bus width 8 (Odd)");
#endif
      CPU.data_width = ActiveBusWidth::EightHigh;
    }
  } else {
// If BHE is inactive, then we can't read an even address. So this must be
// EightLow.
#if DEBUG_BUS
    debugPrintlnColor(ansi::bright_yellow, "Bus width 8 (Even)");
#endif
    CPU.data_width = ActiveBusWidth::EightLow;
  }
}

void cycle() {

  // Resolve data bus from last cycle.
  if (!READ_MRDC_PIN || !READ_IORC_PIN) {
    Controller.getBoard().debugPrintln(DebugType::BUS, "## Resolving data bus ##");
    Controller.writeDataBus(CPU.data_bus, CPU.data_width);
  }

  // First, tick the CPU and increment cycle count
  Controller.tickCpu();
  CYCLE_NUM++;

  CPU.cpuid_counter++;

  // Read the CPU status pins
  CPU.status0 = Controller.readCpuStatusLines();
  CPU.command_bits = Controller.readBusControllerCommandLines();
  // Decode the bus state from the read status. This varies per CPU.
  CPU.bus_state = Controller.decodeBusStatus(CPU.status0);

  // Extract QS0-QS1 queue status
  uint8_t q = (CPU.status0 >> 6) & 0x03;
  CPU.qb = 0xFF;
  CPU.q_ff = false;

  // The ALE signal is issued to inform the motherboard to latch the address bus.
  // The full address is only valid on T1, when ALE is asserted, so if we need to
  // reference the address of the bus cycle later, we must latch it.
  if (Controller.readALEPin()) {
    // ALE signals start of bus cycle, so set cycle to t1.
    #if DEBUG_TSTATE
      debugPrintlnColor(ansi::yellow, "## Setting T-cycle to T1 on ALE");
    #endif

    // This logic doesn't work due to late resolution of Tw states
    // if ((CPU.bus_cycle != T4) && (CPU.bus_cycle != TI)) {
    //   debugPrintlnColor(ansi::red, "## Bad last t-state to enter T1.");
    // }

    CPU.bus_cycle = T1;
    // Address lines are only valid when ALE is high, so latch address now.
    latch_address();
    // Set the data bus width (must happen after latch)
    set_data_bus_width();
    CPU.bus_state_latched = CPU.bus_state;
    CPU.data_bus_resolved = false;
  }

  // We always enter Tw from T3, as we can't tell if the read/write is done yet on T3.
  // Now that we have cycled, we can know if we need to transition to T4, skipping Tw.
  switch (CPU.bus_cycle) {
    case TW:
      // Transition to T4 if read/write signals are complete
      if (is_transfer_done()) {
        CPU.bus_cycle = T4;
        handle_fetch(q);
      }
      break;
    case T4:
      handle_fetch(q);
      CPU.bus_state_latched = PASV;
      break;
    default:
      break;
  }

// Handle queue activity
#if HAVE_QUEUE_STATUS
  if ((q == QUEUE_FIRST) || (q == QUEUE_SUBSEQUENT)) {
    // We fetched a byte from queue last cycle
    if (CPU.queue.len() > 0) {
      CPU.queue.pop(&CPU.qb, &CPU.qt);
      if (q == QUEUE_FIRST) {
        // Set flag for first instruction byte fetched
        CPU.q_ff = true;
        CPU.q_fn = 0;  // First byte of instruction
        CPU.opcode = CPU.qb;
        CPU.mnemonic = get_opcode_str(CPU.opcode, 0, false);
#if DEBUG_INSTR
        if (!IS_GRP_OP(CPU.opcode)) {
          DEBUG_SERIAL.print("INST: ");
          DEBUG_SERIAL.println(CPU.mnemonic);
        } else {
          DEBUG_SERIAL.println("INST: Decoding GRP...");
        }
#endif
      } else {
        if (IS_GRP_OP(CPU.opcode) && CPU.q_fn == 1) {
          CPU.mnemonic = get_opcode_str(CPU.opcode, CPU.qb, true);
#if DEBUG_INSTR
          DEBUG_SERIAL.print("INST: ");
          DEBUG_SERIAL.println(CPU.mnemonic);
#endif
        }
        // Subsequent byte of instruction fetched
        CPU.q_fn++;
      }
    } else {
      // Queue read while queue empty? Bad condition.
      if (CPU.v_state != Reset) {
        // Sometimes we get a spurious queue read signal in Reset.
        // We can safely ignore any queue reads during the Reset state.
        debugPrintlnColor(ansi::bright_red, "## Error: Invalid Queue Length-- ##");
      }
    }
  } else if (q == QUEUE_FLUSHED) {
    // Queue was flushed last cycle.

    // Warn if queue is flushed during CODE cycle.
    if (CPU.bus_state_latched == CODE) {
      DEBUG_SERIAL.print("## FLUSH during CODE fetch! t-state: ");
      switch (CPU.bus_cycle) {
        case TI:
          DEBUG_SERIAL.println("Ti");
          break;
        case T1:
          DEBUG_SERIAL.println("T1");
          break;
        case T2:
          DEBUG_SERIAL.println("T2");
          break;
        case T3:
          DEBUG_SERIAL.println("T3");
          break;
        case TW:
          DEBUG_SERIAL.println("Tw");
          break;
        case T4:
          DEBUG_SERIAL.println("T4");
          break;
      }
    }

    // The queue is flushed once during store program, so we need to adjust s_pc
    // by the length of the queue when it was flushed or else we'll skip bytes
    // of the store program.
    if (CPU.s_pc > 0) {

      if (CPU.s_pc < 4) {
#if DEBUG_STORE
        DEBUG_SERIAL.println("## FLUSHed STORE bytes (early): Reset s_pc");
#endif
        CPU.s_pc = 0;
      } else if (CPU.s_pc >= CPU.queue.len()) {
        uint16_t pc_adjust = (uint16_t)CPU.queue.len();

        if ((pc_adjust & 1) && (CPU.width == BusWidthSixteen)) {
          // If we have an odd queue length and 16-bit fetches, account for one more byte
          //pc_adjust++;
        }
        CPU.s_pc -= pc_adjust;
#if DEBUG_STORE
        DEBUG_SERIAL.print("## FLUSHed STORE bytes: Adjusted s_pc by: ");
        DEBUG_SERIAL.print(pc_adjust);
        DEBUG_SERIAL.print(" new s_pc: ");
        DEBUG_SERIAL.println(CPU.s_pc);
#endif
      } else {
#if DEBUG_STORE
        DEBUG_SERIAL.print("## FLUSHed STORE bytes: Reset s_pc on flush");
#endif
        CPU.s_pc = 0;
      }
    }

    CPU.queue.flush();

#if DEBUG_QUEUE
    DEBUG_SERIAL.println("## Queue Flushed ##");
    DEBUG_SERIAL.print("## PC: ");
    DEBUG_SERIAL.println(CPU.v_pc);
#endif
  }
#endif  // END IF HAVE_QUEUE_STATUS

  uint32_t run_address = 0;

  if (!READ_MWTC_PIN || !READ_IOWC_PIN) {
    // CPU is writing to the data bus, latch value
    CPU.data_bus = data_bus_read(CPU.data_width);
  }

  // Handle state machine
  switch (CPU.v_state) {

    case Reset:
      // We are executing the CPU reset routine.
      // Nothing to do here.
      break;
    case CpuId:
      // We are executing the CPU ID routine.
      handle_cpuid_state(q);
      break;

    case CpuSetup:
      handle_cpu_setup_state();
      break;

    case JumpVector:
      // We are executing the initial jump from the reset vector FFFF:0000.
      // This is to avoid wrapping effective address during load procedure.
      handle_jump_vector_state(q);
      break;

    case Load:
      // We are executing the register load routine.
      if (CPU.cpu_type == CpuType::i80286) {
        // 286 just uses LOADALL instead of a load program. 
        handle_loadall_286();
      } else {
        handle_load_state(q);
      }
      
      break;

    case LoadDone:
      // LoadDone is triggered by the queue flush following the jump in Load.
      // We wait for the next ALE and begin Execute.
      handle_load_done_state();
      break;

    case Prefetch:
      // We are executing the prefetch routine.
      // Currently placeholder.
      break;

    case EmuEnter:
      // We are executing the BRKEM routine.
      handle_emu_enter_state(q);
      break;

    // Unlike in run_program, the Execute state in cpu_server is entirely interactive based on
    // commands from the client.
    // This is to support interception of memory reads & writes as instructions execute and to allow
    // the client to query CPU state as it wishes per cpu cycle.
    // When done in the Execute state, a cpu client can end execution by:
    //  - Executing an ExecuteFinalize command.
    //    This is typically done when a CODE fetch occurs past the end of the provided program, although
    //    other end conditions are possible.
    //  - Executing a HALT
    //  - Asserting NMI before the end of the instruction
    case Execute:

      if ((!READ_MRDC_PIN || !READ_IORC_PIN) && CPU.bus_cycle == WRITE_CYCLE) {
        // CPU is reading from data bus. We assume that the client has called CmdWriteDataBus to set
        // the value of CPU.data_bus. Write it.
        //data_bus_write(CPU.data_bus, CPU.data_width);
        Controller.writeDataBus(CPU.data_bus, CPU.data_width);
        //CPU.data_bus_resolved = true;

        #if DEBUG_EXECUTE
          debugPrintColor(ansi::green, "## EXECUTE: Wrote bus: ");
          debugPrintlnColor(ansi::green, CPU.data_bus, 16);
        #endif

        if ((CPU.bus_state_latched == CODE) && (CPU.prefetching_store)) {
          //CPU.s_pc++;
          #if DEBUG_STORE
            debugPrintColor(ansi::yellow, "## EXECUTE: Wrote STORE PGM BYTE to bus: ");
            debugPrintColor(ansi::yellow, CPU.data_bus, 16);
            debugPrintColor(ansi::yellow, " new s_pc: ");
            debugPrintlnColor(ansi::yellow, CPU.s_pc);
          #endif
        }
      }

      if (CPU.bus_state == HALT) {
        #if DEBUG_EXECUTE
          debugPrintlnColor(ansi::green, "## EXECUTE: Detected HALT - Setting NMI high to end program execution.");
        #endif
        Controller.writePin(OutputPin::Nmi, true);
        break;
      }

      if (READ_NMI_PIN) {
        // Use checkpoint "1" to specify that NMI has been detected. This just prevents the debug message from
        // printing every cycle after NMI.
        if (CPU.nmi_checkpoint == 0) {
          #if DEBUG_EXECUTE
            debugPrintlnColor(ansi::green, "## EXECUTE: Detected NMI high - Execute will end at IVT fetch.");
          #endif
          CPU.nmi_checkpoint = 1;
        }
        if (!CPU.data_bus_resolved && !READ_MRDC_PIN) {
          // NMI is active and CPU is reading memory.  Let's check if it is the NMI handler.
          if (CPU.address_latch == 0x00008) {
#if DEBUG_EXECUTE
            debugPrintlnColor(ansi::green, "## EXECUTE: NMI high and fetching NMI IVT. Entering ExecuteFinalize...");
#endif
            CPU.nmi_terminate = true;
            change_state(ExecuteFinalize);
          }
        }
      }

      break;

    // Since Execute is an interactive state where the client controls the cpu_server, we need to be able
    // to transition safely from Execute to Store.
    // ExecuteFinalize state feeds the CPU STORE program bytes flagged with DATA_PROGRAM_END and
    // transitions to Store when one of those bytes is fetched as the first byte of an instruction.
    //
    // If NMI is high during ExecuteFinalize, reading the NMI vector completely will reset the store program
    // PC and also transition to ExecuteDone. This is the newer method of ending program execution that
    // supports the 80186 without queue status lines.
    case ExecuteFinalize:

      if (READ_NMI_PIN) {
        if (!CPU.data_bus_resolved && !READ_MRDC_PIN) {
          // NMI is active and CPU is reading memory.  Let's check if it is the NMI handler.
          if (CPU.address_latch == 0x00008) {
            CPU.nmi_checkpoint = 1;
          }

          if (CPU.bus_state_latched == CODE) {
            // CPU is reading CODE in ExecuteFinalize with NMI high.
            // This should hopefully be at the address of the NMI vector, so we can enter ExecuteDone
            run_address = calc_flat_address(STORE_SEG, 0);
            if (CPU.address_latch == run_address) {
#if DEBUG_EXECUTE
              debugPrintlnColor(ansi::green, "## EXECUTE_FINALIZE: Fetch at STORE_SEG.");
#endif
              change_state(ExecuteDone);
            }
          } else if (CPU.nmi_checkpoint > 0 && CPU.v_pc < sizeof(NMI_VECTOR)) {
            // Feed the CPU the NMI vector.
            CPU.data_bus = read_program(NMI_VECTOR, sizeof NMI_VECTOR, &CPU.v_pc, CPU.address_latch, CPU.data_width);
#if DEBUG_EXECUTE
            debugPrintColor(ansi::green, "## EXECUTE_FINALIZE: Feeding CPU reset vector data: ");
            debugPrintColor(ansi::green, CPU.data_bus, 16);
            debugPrintColor(ansi::green, " new v_pc: ");
            debugPrintlnColor(ansi::green, CPU.v_pc);
#endif
            CPU.data_bus_resolved = true;
            data_bus_write(CPU.data_bus, CPU.data_width);

            if (CPU.nmi_checkpoint == 1 && CPU.address_latch == 0x0000A) {
#if DEBUG_EXECUTE
              debugPrintlnColor(ansi::green, "## EXECUTE_FINALIZE: Read of NMI IVT with NMI pin high - Resetting STORE PC");
#endif
              CPU.nmi_checkpoint = 2;
              CPU.data_bus_resolved = true;
              CPU.s_pc = 0;
            }
            break;
          }
        }

        if (!CPU.data_bus_resolved && !READ_MWTC_PIN && CPU.nmi_checkpoint > 1) {
          // NMI is active and CPU is writing memory. Probably to stack.
          write_buffer(NMI_STACK_BUFFER, &CPU.nmi_buf_cursor, CPU.data_bus, CPU.address_latch, CPU.data_width);
#if DEBUG_EXECUTE
          debugPrintColor(ansi::green, "## EXECUTE_FINALIZE: Stack write: ");
          debugPrintColor(ansi::green, CPU.data_bus, 16);
          debugPrintColor(ansi::green, " New buf cursor: ");
          debugPrintlnColor(ansi::green, CPU.nmi_buf_cursor);
#endif
          CPU.data_bus_resolved = true;
        }
      }

      if (!READ_MRDC_PIN && CPU.bus_state == PASV) {
        // CPU is reading (MRDC active-low)
        if (CPU.bus_state_latched == CODE) {
          // CPU is fetching code

          // Since client does not cycle the CPU in this state, we have to fetch from the
          // STORE or EMU_EXIT program ourselves
          CPU.data_bus = read_program(CPU.program, CPU.program_len, CPU.program_pc, CPU.address_latch, CPU.data_width);
          CPU.data_type = QueueDataType::ProgramEnd;
          data_bus_write(CPU.data_bus, CPU.data_width);
#if DEBUG_STORE
          debugPrintColor(ansi::green, "## EXECUTE_FINALIZE: Wrote next PGM word to bus: ");
          debugPrintColor(ansi::green, CPU.data_bus, 16);
          debugPrintColor(ansi::green, " new s_pc: ");
          debugPrintlnColor(ansi::green, CPU.s_pc);
#endif
        } else {
          data_bus_write(CPU.data_bus, CPU.data_width);
        }
      }

      if (CPU.q_ff && (CPU.qt == QueueDataType::ProgramEnd)) {
        // We read a flagged NOP, meaning the previous instruction has completed and it is safe to
        // execute the Store routine.
        if (CPU.in_emulation) {
          change_state(EmuExit);
        } else {
          change_state(ExecuteDone);
        }
      }
      break;

    case EmuExit:
      if (!READ_MRDC_PIN) {
        // CPU is reading (MRDC active-low)
        if ((CPU.bus_state_latched == CODE) && (CPU.bus_state == PASV)) {
          // CPU is doing code fetch
          if (CPU.s_pc < sizeof EMU_EXIT_PROGRAM) {
            // Read code byte from EmuExit program
            CPU.data_bus = read_program(EMU_EXIT_PROGRAM, sizeof EMU_EXIT_PROGRAM, &CPU.s_pc, CPU.address_latch, CPU.data_width);
#if DEBUG_EMU
            DEBUG_SERIAL.print("## EMUEXIT: fetching byte: ");
            DEBUG_SERIAL.print(CPU.data_bus, 16);
            DEBUG_SERIAL.print(" new s_pc: ");
            DEBUG_SERIAL.println(CPU.s_pc);
#endif
            CPU.data_type = QueueDataType::Program;
          } else {
            CPU.data_bus = OPCODE_DOUBLENOP;
            CPU.data_type = QueueDataType::ProgramEnd;
          }
          data_bus_write(CPU.data_bus, CPU.data_width);
        }

        if ((CPU.bus_state_latched == MEMR) && (CPU.bus_state == PASV)) {
          // CPU is doing memory read
          // This will occur when RETEM pops IP, CS and Flags from the stack.

          if (CPU.width == BusWidthEight) {
            // Stack values will be read in two operations
            if (CPU.stack_r_op_ct == 0) {

            } else if (CPU.stack_r_op_ct == 1) {

            } else if (CPU.stack_r_op_ct == 2) {
#if DEBUG_EMU
              DEBUG_SERIAL.println("## Reading RETEM CS pop (1/2)! ##");
              DEBUG_SERIAL.println(CPU.load_regs.cs);
#endif
              // Write the low byte of CS to the data bus
              data_bus_set_byte((uint8_t)(CPU.load_regs.cs));
            } else if (CPU.stack_r_op_ct == 3) {
#if DEBUG_EMU
              DEBUG_SERIAL.println("## Reading RETEM CS pop (2/2)! ##");
              DEBUG_SERIAL.println(CPU.load_regs.cs);
#endif
              // Write the high byte of CS to the data bus
              data_bus_set_byte((uint8_t)(CPU.load_regs.cs >> 8));
            } else if (CPU.stack_r_op_ct == 4) {
#if DEBUG_EMU
              DEBUG_SERIAL.println("## Reading RETEM flag pop (1/2)! ##");
#endif
              // Write the low byte of flags to the data bus
              data_bus_set_byte((uint8_t)(CPU.pre_emu_flags));
            } else if (CPU.stack_r_op_ct == 5) {
#if DEBUG_EMU
              DEBUG_SERIAL.println("## Reading RETEM flag pop (2/2)! ##");
#endif
              // Write the high byte of flags to the data bus
              data_bus_set_byte((uint8_t)(CPU.pre_emu_flags >> 8));
              // Exit emulation mode
              CPU.in_emulation = false;
              change_state(ExecuteFinalize);
            } else {
              // Not flags, just write 0's so we jump back to CS:IP 0000:0000
              CPU.data_bus = 0;
            }
            CPU.stack_r_op_ct++;
          } else {
            // Sixteen-bit data bus

            if (CPU.stack_r_op_ct == 0) {
// IP is read in one operation
#if DEBUG_EMU
              DEBUG_SERIAL.println("## Reading RETEM IP pop! ##");
#endif
              CPU.data_bus = 0;
            } else if (CPU.stack_r_op_ct == 1) {
// CS is read in one operation
#if DEBUG_EMU
              DEBUG_SERIAL.println("## Reading RETEM CS pop! ##");
              DEBUG_SERIAL.println(CPU.load_regs.cs);
#endif
              // We can restore CS from the loaded registers since CS cannot be modified in 8080 emulation mode
              CPU.data_bus = CPU.load_regs.cs;
            } else if (CPU.stack_r_op_ct == 2) {
// Flags will be read in one operation
#if DEBUG_EMU
              DEBUG_SERIAL.println("## Reading RETEM Flag pop! ##");
#endif
              // CPU is writing to the data bus, latch value
              CPU.data_bus = CPU.pre_emu_flags;
              // Exit emulation mode
              CPU.in_emulation = false;
              change_state(ExecuteFinalize);
            }
            CPU.stack_r_op_ct++;
          }
          data_bus_write(CPU.data_bus, CPU.data_width);
        }
      }

      if (!READ_MWTC_PIN && (CPU.bus_state_latched == MEMW) && (CPU.bus_state == PASV)) {
        // CPU is writing. This should only happen during EmuExit when we PUSH PSW
        // to save the 8080 flags.
        if (CPU.width == BusWidthEight) {
          // 8-bit data bus
          if (CPU.stack_w_op_ct == 0) {
// Flags will be in first byte written (second byte will be AL)
#if DEBUG_EMU
            DEBUG_SERIAL.println("## Capturing PUSH PSW stack write! ##");
#endif
            CPU.emu_flags = (uint8_t)CPU.data_bus;
          }
          CPU.stack_w_op_ct++;
        } else {
          // 16-bit data bus
          if (CPU.stack_w_op_ct == 0) {
// Flags were pushed in one operation.
#if DEBUG_EMU
            DEBUG_SERIAL.println("## Capturing PUSH PSW stack write! ##");
#endif
            CPU.emu_flags = (uint8_t)CPU.data_bus;
          }
          CPU.stack_w_op_ct++;
        }
      }

      break;

    case ExecuteDone:
      // We sit in ExecuteDone state until the client requests a Store operation.
      // The client should not cycle the CPU in this state.
      if (!READ_MRDC_PIN && CPU.bus_state == PASV) {
        // CPU is reading (MRDC active-low)
        data_bus_write(CPU.data_bus, CPU.data_width);

        if ((CPU.bus_state_latched == CODE) && (CPU.prefetching_store)) {
          // Since client does not cycle the CPU in this state, we have to fetch from
          // STORE program ourselves

          CPU.data_bus = read_program(CPU.program, CPU.program_len, CPU.program_pc, CPU.address_latch, CPU.data_width);
          //CPU.data_bus = STORE_PROGRAM[CPU.s_pc++];
          CPU.data_type = QueueDataType::ProgramEnd;
          data_bus_write(CPU.data_bus, CPU.data_width);
#if DEBUG_STORE
          DEBUG_SERIAL.print("## STORE: Wrote STORE PGM BYTE to bus (in EXECUTE_DONE): ");
          DEBUG_SERIAL.print(CPU.data_bus, 16);
          DEBUG_SERIAL.print(" new s_pc: ");
          DEBUG_SERIAL.println(CPU.s_pc);
#endif
        } else {
          DEBUG_SERIAL.println("## Invalid condition: ExecuteDone without loading STORE");
          data_bus_write(CPU.data_bus, CPU.data_width);
        }
      }
      break;

    case Store:
      // We are executing the Store program.
      if (!READ_MRDC_PIN && CPU.bus_state == PASV) {
        // CPU is reading

        if (CPU.bus_state_latched == CODE) {
          // CPU is doing code fetch
          if (CPU.s_pc < CPU.program_len) {
            // Read code byte from store program
            //CPU.data_bus = STORE_PROGRAM[CPU.s_pc++];
            CPU.data_bus = read_program(CPU.program, CPU.program_len, CPU.program_pc, CPU.address_latch, CPU.data_width);
#if DEBUG_STORE
            debugPrintColor(ansi::magenta, "## STORE: fetching code: ");
            debugPrintColor(ansi::magenta, CPU.data_bus, 16);
            debugPrintColor(ansi::magenta, " new s_pc: ");
            debugPrintlnColor(ansi::magenta, CPU.s_pc);
#endif
            CPU.data_type = QueueDataType::Program;

          } else {
            CPU.data_bus = OPCODE_DOUBLENOP;
            CPU.data_type = QueueDataType::ProgramEnd;
          }
        } else {
          // CPU is reading something else. Stack?
          CPU.data_bus = read_program(NMI_STACK_BUFFER, sizeof NMI_STACK_BUFFER, &CPU.nmi_buf_cursor, CPU.address_latch, CPU.data_width);
#if DEBUG_STORE
          debugPrintColor(ansi::magenta, "## STORE: Reading from stack: ");
          debugPrintColor(ansi::magenta, CPU.data_bus, 16);
          debugPrintColor(ansi::magenta, " new cursor: ");
          debugPrintlnColor(ansi::magenta, CPU.nmi_buf_cursor);
#endif
        }
        data_bus_write(CPU.data_bus, CPU.data_width);
      }

      // CPU is writing to memory address - this should only happen during readback when
      // the flags register is pushed to the stack (The only way to read the full flags)
      if (!READ_MWTC_PIN && CPU.bus_state != PASV) {
        CPU.data_bus = data_bus_read(CPU.data_width);

        // Store program sets up SS:SP as 0:4, so write should be to the first four memory
        // addresses, for pushing IP and FLAGS.
        if (CPU.address_latch < 0x00004) {

#if DEBUG_STORE
          debugPrintlnColor(ansi::magenta, "## STORE Stack Push");
#endif

          // Write flags and IP to the register struct
          if (CPU.data_width == ActiveBusWidth::EightLow) {
#if DEBUG_EMU
            DEBUG_SERIAL.print("## 8-bit flag read ##");
#endif
            *CPU.readback_p = (uint8_t)CPU.data_bus;
            CPU.readback_p++;
          } else if (CPU.data_width == ActiveBusWidth::EightHigh) {
            // We shouldn't have unaligned stack access during STORE. Something has gone wrong.
            debugPrintlnColor(ansi::bright_red, "## Bad Data Bus Width during Store: EightHigh");
          } else {
            // 16-bit data bus
            if ((CPU.address_latch == 0x00002) && (CPU.do_emulation)) {
              // We ran a program in 8080 emulation. We want to substitute the flags
              // captured in 8080 mode for the native flags now.
              CPU.data_bus = (CPU.data_bus & 0xFF00) | (uint16_t)CPU.emu_flags;
#if DEBUG_EMU
              debugPrintColor(ansi::magenta, "## Substituting 8080 flags in stack read: ");
              debugPrintlnColor(ansi::magenta, CPU.data_bus, 16);
#endif
            }

#if DEBUG_EMU
            ptrdiff_t diff = (uint8_t *)&CPU.post_regs.flags - CPU.readback_p;
#endif            
            *((uint16_t *)CPU.readback_p) = CPU.data_bus;
            CPU.readback_p += 2;

#if DEBUG_EMU
            uint16_t *flags_ptr = (uint16_t *)&CPU.post_regs.flags;
            debugPrintColor(DEBUG_STORE_COLOR, "## New flags are: ");
            debugPrintColor(DEBUG_STORE_COLOR, *flags_ptr, 16);
            debugPrintColor(DEBUG_STORE_COLOR, "## Readback ptr diff: ");
            debugPrintlnColor(DEBUG_STORE_COLOR, diff);
#endif
          }
        } else {
          // We shouldn't be writing to any other addresses, something wrong happened
          if (CPU.address_latch == 0x00004) {
            debugPrintlnColor(ansi::bright_red, "## TRAP detected in Store operation! Invalid flags?");
          }

          debugPrintColor(ansi::bright_red, "## INVALID STORE MEMORY WRITE: ");
          debugPrintlnColor(ansi::bright_red, CPU.address_latch, HEX);
          set_error("Invalid store memory write");
          // TODO: handle error gracefully
        }
#if DEBUG_STORE
        debugPrintColor(DEBUG_STORE_COLOR, "## STORE: memory write: ");
        debugPrintlnColor(DEBUG_STORE_COLOR, CPU.data_bus, HEX);
#endif
      }

      // CPU is writing to IO address - this indicates we are saving a register value.
      // We structured the register struct in the right order, so we can overwrite it
      // directly.
      if (!READ_IOWC_PIN) {
#if DEBUG_STORE
        debugPrintlnColor(DEBUG_STORE_COLOR, "## STORE: IO Write");
#endif
        if (CPU.address_latch == 0xFD) {
// Write to 0xFD indicates end of store procedure.

// Adjust IP by offset of CALL instruction.
#if DEBUG_STORE
          DEBUG_SERIAL.print("## Unadjusted IP: ");
          DEBUG_SERIAL.println(CPU.post_regs.ip, HEX);
#endif
          //CPU.post_regs.ip -= 0x24;
          //CPU.post_regs.ip -= (0x24 + 6); // added 6 NOPs to start of STORE program

          change_state(StoreDone);
        } else {
          CPU.data_bus = data_bus_read(CPU.data_width);

          if (CPU.data_width == ActiveBusWidth::EightLow) {
            *CPU.readback_p = (uint8_t)CPU.data_bus;
            CPU.readback_p++;
          } else if (CPU.data_width == ActiveBusWidth::EightHigh) {
            DEBUG_SERIAL.println("## Bad Data Bus Width during Store: EightHigh");
          } else {
            *(uint16_t *)CPU.readback_p = CPU.data_bus;
            CPU.readback_p += 2;
          }

#if DEBUG_STORE
          debugPrintColor(ansi::bright_magenta, "## STORE: IO write: ");
          debugPrintlnColor(ansi::bright_magenta, CPU.data_bus, HEX);
#endif
        }
      }
      break;

    case StoreDone:
      // We are done with the Store program.
      break;
    case Done:
      break;
  }

  // Print instruction state if tracing is enabled
  switch (CPU.v_state) {
    case Reset:
#if TRACE_RESET
      print_cpu_state();
#endif
      break;
    case CpuId:
#if TRACE_ID
      print_cpu_state();
#endif
      break;
    case CpuSetup:
#if TRACE_SETUP
      print_cpu_state();
#endif
      break;
    case JumpVector:
#if TRACE_VECTOR
      print_cpu_state();
#endif
      break;
    case Load:  // FALLTHROUGH
    case LoadDone:
#if TRACE_LOAD
      print_cpu_state();
#endif
      break;
    case Prefetch:
#if TRACE_PREFETCH
      print_cpu_state();
#endif
      break;
    case EmuEnter:
#if TRACE_EMU_ENTER
      print_cpu_state();
#endif
      break;
    case EmuExit:
#if TRACE_EMU_EXIT
      print_cpu_state();
#endif
      break;
    case Execute:
#if TRACE_EXECUTE
      print_cpu_state();
#endif
      break;
    case ExecuteDone:  // FALLTHROUGH
    case ExecuteFinalize:
#if TRACE_FINALIZE
      print_cpu_state();
#endif
      break;
    case Done: // FALLTHROUGH
    case StoreDone:  // FALLTHROUGH
    case Store:
#if TRACE_STORE
      print_cpu_state();
#endif
      break;
  }

  // Transition to next T-state.
  CPU.last_bus_cycle = CPU.bus_cycle;
  CPU.bus_cycle = Controller.getNextCycle(CPU.bus_cycle, CPU.bus_state, CPU.bus_state_latched);
}

void handle_fetch(uint8_t q) {
  // Did we complete a code fetch? If so, increment queue len
  if (CPU.bus_state_latched == CODE) {
    //DEBUG_SERIAL.print("## T4 of CODE fetch. Q is: ");
    //DEBUG_SERIAL.println(q);

#if DEBUG_QUEUE
    debugPrintlnColor(DEBUG_QUEUE_COLOR, "## QUEUE: T4 of code fetch!");
#endif

    if (q == QUEUE_FLUSHED) {
#if DEBUG_QUEUE
      debugPrintlnColor(DEBUG_QUEUE_COLOR, "## Queue flush during T4.");
#endif
      if (CPU.queue.have_room(CPU.data_width)) {
        CPU.queue.push(CPU.data_bus, CPU.data_type, CPU.data_width);
      } else {
        // No room for fetch - this shouldn't happen!
        debugPrintlnColor(ERROR_COLOR, "## Error: Invalid Queue Length++ ##");
      }
    } else {
      if (CPU.queue.have_room(CPU.data_width)) {
#if DEBUG_QUEUE
        debugPrintColor(DEBUG_QUEUE_COLOR, "## QUEUE: T4, Pushing data bus to queue: ");
        debugPrintlnColor(DEBUG_QUEUE_COLOR, CPU.data_bus, HEX);
#endif
        CPU.queue.push(CPU.data_bus, CPU.data_type, CPU.data_width);
      } else {
        // Shouldn't be here
        debugPrintlnColor(ansi::bright_red, "## Error: Invalid Queue Length++ ##");
      }
    }
  }
}
void handle_cpuid_state(uint8_t q) {

  if (q == QUEUE_FIRST) {
    if (CPU.cpuid_queue_reads == 0) {
#if DEBUG_ID
      debugPrintlnColor(DEBUG_ID_COLOR, "## CPUID: Starting CPUID counter! ##");
#endif
      CPU.cpuid_counter = 0;
    } else if (CPU.cpuid_queue_reads == 1) {
#if DEBUG_ID
      debugPrintColor(DEBUG_ID_COLOR, "## CPUID: counter stopped at: ");
      debugPrintColor(DEBUG_ID_COLOR, CPU.cpuid_counter);
      debugPrintlnColor(DEBUG_ID_COLOR, " ##");
#endif
      detect_cpu_type(CPU.cpuid_counter);
    }
    CPU.cpuid_queue_reads++;
  }

  // Change state after we have executed a minimum number of instructions
  if (CPU.cpuid_queue_reads > 4) {
#if USE_SETUP_PROGRAM
    change_state(CpuSetup);
#else
    change_state(JumpVector);
#endif
  }

  if (!READ_MRDC_PIN) {
    // CPU is reading (MRDC active-low)
    if (CPU.bus_state == CODE) {
      // We are reading a code byte

      // Feed the program if we haven't this bus cycle.
      if (!CPU.data_bus_resolved && (CPU.v_pc < sizeof CPUID_PROGRAM)) {
        // Feed CPU ID instruction to CPU.
        CPU.data_bus = read_program(CPUID_PROGRAM, sizeof CPUID_PROGRAM, &CPU.v_pc, CPU.address_latch, CPU.data_width);
        CPU.data_type = QueueDataType::Program;
        data_bus_write(CPU.data_bus, CPU.data_width);

#if DEBUG_ID
        debugPrintColor(DEBUG_ID_COLOR, "## CPUID: Writing CPUID program to bus: ");
        debugPrintColor(DEBUG_ID_COLOR, CPU.data_bus, 16);
        debugPrintColor(DEBUG_ID_COLOR, " new pc: ");
        debugPrintColor(DEBUG_ID_COLOR, CPU.v_pc);
        debugPrintColor(DEBUG_ID_COLOR, "/");
        debugPrintlnColor(DEBUG_ID_COLOR, sizeof CPUID_PROGRAM);
#endif
      }
    }
  }

  if (!READ_MWTC_PIN && READ_TEST_PIN) {
    // FPU is writing to bus
    if (CPU.data_bus == 0x03FF) {
// Have an 8087!
#if DEBUG_ID
      debugPrintlnColor(DEBUG_ID_COLOR, "## CPUID: Detected 8087 status word write!");
#endif
      // This doesn't do anything atm besides set an 8087
      detect_fpu_type();
    }
  }
}

// Handle the CpuSetup state.
// This state is not for register loading, but to handle CPUs that require some configuration before
// they can be used by the cpu_server. Currently the only CPUs that need setup are the 188 and 186
// which have to have interrupts enabled via the PCB as they are masked off at RESET.
void handle_cpu_setup_state() {
  if (!READ_MRDC_PIN) {
    // CPU is reading (MRDC active-low)
    if (CPU.bus_state == CODE) {
      // We are reading a code byte

      // Feed the program if we haven't this bus cycle.
      if (!CPU.data_bus_resolved) {
        if (CPU.v_pc < sizeof SETUP_PROGRAM) {
          // Feed SETUP_PROGRAM instruction to CPU.
          CPU.data_bus = read_program(SETUP_PROGRAM, sizeof SETUP_PROGRAM, &CPU.v_pc, CPU.address_latch, CPU.data_width);
          CPU.data_type = QueueDataType::Program;
          data_bus_write(CPU.data_bus, CPU.data_width);
        } else {
          // Ran out of program, so return NOP. Doesn't matter what we feed
          // as queue will be reset.
          CPU.data_bus = read_nops(CPU.data_width);
          CPU.data_type = QueueDataType::ProgramEnd;
        }
#if DEBUG_SETUP
        debugPrintColor(ansi::cyan, "## Writing SETUP_PROGRAM program to bus: ");
        debugPrintlnColor(ansi::cyan, CPU.data_bus, 16);
#endif
        CPU.data_bus_resolved = true;
        data_bus_write(CPU.data_bus, CPU.data_width);
      }
    }
  }

  if (Controller.readALEPin()) {
    // Jump is finished on first address latch of LOAD_SEG:0
    uint32_t dest = calc_flat_address(LOAD_SEG, 0);
    if (dest == CPU.address_latch) {
#if DEBUG_SETUP
      debugPrintColor(ansi::cyan, "## ALE at LOAD_SEG. Transitioning to Load state. SEG: ");
      debugPrintlnColor(ansi::cyan, CPU.address_latch, 16);
#endif
      // Transition to Load state.
      change_state(Load);
    }
  }
}

void handle_jump_vector_state(uint8_t q) {
  if (!READ_MRDC_PIN) {
    // CPU is reading (MRDC active-low)
    if (CPU.bus_state_latched == CODE) {
      // We are reading a code byte.
      // If the data bus hasn't been resolved this m-cycle, feed in the JUMP_VECTOR program
      if (!CPU.data_bus_resolved) {
        if (CPU.v_pc < sizeof JUMP_VECTOR) {
          // Feed jump instruction to CPU
          CPU.data_bus = read_program(JUMP_VECTOR, sizeof JUMP_VECTOR, &CPU.v_pc, CPU.address_latch, CPU.data_width);
          CPU.data_type = QueueDataType::Program;
        } else {
          // Ran out of program, so return NOP. Doesn't matter what we feed
          // as queue will be reset.
          CPU.data_bus = read_nops(CPU.data_width);
          CPU.data_type = QueueDataType::ProgramEnd;
        }
#if DEBUG_VECTOR
        debugPrintColor(DEBUG_VECTOR_COLOR, "## Writing JUMP_VECTOR program to bus: ");
        debugPrintColor(DEBUG_VECTOR_COLOR, CPU.data_bus, 16);
        debugPrintColor(DEBUG_VECTOR_COLOR, " new pc: ");
        debugPrintlnColor(DEBUG_VECTOR_COLOR, CPU.v_pc);
#endif
        CPU.data_bus_resolved = true;
        
        Controller.writeDataBus(CPU.data_bus, CPU.data_width);
        //data_bus_write(CPU.data_bus, CPU.data_width);
      }
    }
  }

  if (Controller.readALEPin()) {
    // Jump is finished on first address latch of LOAD_SEG:0
    uint32_t dest = calc_flat_address(LOAD_SEG, 0);
    if (dest == CPU.address_latch) {
#if DEBUG_VECTOR
      debugPrintColor(ansi::cyan, "## ALE at LOAD_SEG. Transitioning to Load state. SEG: ");
      debugPrintlnColor(ansi::cyan, CPU.address_latch, 16);
#endif
      // Transition to Load state.
      change_state(Load);
    }
  }
}

void handle_loadall_286() {
  if (!READ_MRDC_PIN) {
    // CPU is reading (MRDC active-low)
    if (CPU.bus_state_latched == CODE) {
      // We are reading a code byte

      // If we haven't resolved the data bus this bus cycle...
      if (!CPU.data_bus_resolved) {
        if (CPU.v_pc < CPU.program_len) {
          // Feed load program to CPU
          CPU.data_bus = read_program(CPU.program, CPU.program_len, CPU.program_pc, CPU.address_latch, CPU.data_width);
          CPU.data_type = QueueDataType::Program;
        } else {
          // Ran out of program, so return NOP. JMP cs:ip will actually fetch once before SUSP,
          // so we wil see this NOP prefetched.
          CPU.data_bus = OPCODE_DOUBLENOP;
          CPU.data_type = QueueDataType::ProgramEnd;
          //change_state(LoadDone);
        }
#if DEBUG_LOAD
        debugPrintColor(ansi::green, "## Writing LOAD program to bus: ");
        debugPrintlnColor(ansi::green, CPU.data_bus, 16);
#endif
        Controller.writeDataBus(CPU.data_bus, CPU.data_width);
        CPU.data_bus_resolved = true;
      }
    }

    if (CPU.bus_state_latched == MEMR) {
      // We are reading a memory word
      if ((CPU.address_latch >= LOADALL286_ADDRESS) && (CPU.address_latch < (LOADALL286_ADDRESS + sizeof CPU.loadall_regs))) {
        size_t idx = (CPU.address_latch - LOADALL286_ADDRESS) / 2;
        uint16_t *word_ptr = ((uint16_t *)&CPU.loadall_regs);
        CPU.data_bus = word_ptr[idx];
        Controller.getBoard().debugPrint(DebugType::LOAD, "## LOADALL_286: Writing LOADALL word to bus: ");
        Controller.getBoard().debugPrintln(DebugType::LOAD, CPU.data_bus, 16);
        Controller.writeDataBus(CPU.data_bus, CPU.data_width);
        CPU.data_bus_resolved = true;
      } else {
        // Unexpected read out of LOADALL range.
        Controller.getBoard().debugPrintln(DebugType::ERROR, "## LOADALL_286: INVALID MEM READ ##");
      }
    }
  }

  // We can't tell when the queue flushed but we can see the initial code fetch at the new CS:IP.
  // We don't need to enter LoadDone in this case, we can jump directly to Execute as all LoadDone does is wait
  // for ALE. (TODO: Should this just be the primary way we leave Load?)
  uint32_t base_address = (static_cast<uint32_t>(CPU.loadall_regs.CS_DESC.addr_hi)) << 16 | 
                          (static_cast<uint32_t>(CPU.loadall_regs.CS_DESC.addr_lo));

  uint32_t run_address = base_address + static_cast<uint32_t>(CPU.loadall_regs.IP);
  
  if (CPU.address_latch == run_address) {
#if DEBUG_LOAD
    debugPrintColor(ansi::green, "## LOADALL_286: Detected jump to new CS:IP to trigger transition into Execute");
    debugPrintlnColor(ansi::green, CPU.data_bus, 16);
#endif
    change_state(Execute);
  }
}

void handle_load_state(uint8_t q) {
  if (!READ_MRDC_PIN) {
    // CPU is reading (MRDC active-low)
    if (CPU.bus_state_latched == CODE) {
      // We are reading a code byte

      // If we haven't resolved the data bus this bus cycle...
      if (!CPU.data_bus_resolved) {
        if (CPU.v_pc < CPU.program_len) {
          // Feed load program to CPU
          CPU.data_bus = read_program(CPU.program, CPU.program_len, CPU.program_pc, CPU.address_latch, CPU.data_width);
          CPU.data_type = QueueDataType::Program;
        } else {
          // Ran out of program, so return NOP. JMP cs:ip will actually fetch once before SUSP,
          // so we wil see this NOP prefetched.

#if (DATA_BUS_SIZE == 1)
          CPU.data_bus = OPCODE_NOP;
#else
          CPU.data_bus = OPCODE_DOUBLENOP;
#endif

          CPU.data_type = QueueDataType::ProgramEnd;
          //change_state(LoadDone);
        }
#if DEBUG_LOAD
        debugPrintColor(ansi::green, "## Writing LOAD program to bus: ");
        debugPrintlnColor(ansi::green, CPU.data_bus, 16);
#endif
        
        Controller.writeDataBus(CPU.data_bus, CPU.data_width);
        CPU.data_bus_resolved = true;
      }
    }

    if (CPU.cpu_type != CpuType::i80286) {
      if (CPU.bus_state == MEMR) {
        // We are reading a memory byte
        // This should only occur during Load when flags are popped from 0:0
        if (CPU.address_latch < 0x00002) {
          // First two bytes of LOAD_PROGRAM were patched with flags
          uint16_t dummy_pc = (uint16_t)CPU.address_latch;
          CPU.data_bus = read_program(LOAD_PROGRAM, sizeof LOAD_PROGRAM, &dummy_pc, CPU.address_latch, CPU.data_width);
          CPU.data_type = QueueDataType::Program;
          Controller.writeDataBus(CPU.data_bus, CPU.data_width);
          CPU.data_bus_resolved = true;
        } else {
          // Unexpected read above address 0x00001
          debugPrintlnColor(ansi::bright_red, "## INVALID MEM READ DURING LOAD ##");
        }
      }
    }

  }

#if HAVE_QUEUE_STATUS
  if (q == QUEUE_FLUSHED) {
#if DEBUG_LOAD
    debugPrintlnColor(ansi::green, "## Detected queue flush to trigger transition into LoadDone");
#endif
    // Queue flush after final jump triggers next state.
    change_state(LoadDone);
  }
#else
  // We can't tell when the queue flushed but we can see the initial code fetch at the new CS:IP.
  // We don't need to enter LoadDone in this case, we can jump directly to Execute as all LoadDone does is wait
  // for ALE. (TODO: Should this just be the primary way we leave Load?)
  uint32_t run_address = calc_flat_address(CPU.load_regs.cs, CPU.load_regs.ip);
  if (CPU.address_latch == run_address) {
#if DEBUG_LOAD
    debugPrintColor(ansi::green, "## 186: Detected jump to new CS:IP to trigger transition into Execute");
    debugPrintlnColor(ansi::green, CPU.data_bus, 16);
#endif
    change_state(Execute);
  }
#endif
}

void handle_load_done_state() {
#if DEBUG_LOAD_DONE
  DEBUG_SERIAL.print("LoadDone: Controller.readALEPin()=");
  DEBUG_SERIAL.print(Controller.readALEPin());
  DEBUG_SERIAL.print(" CPU.bus_state=");
  DEBUG_SERIAL.println(BUS_STATE_STRINGS[(size_t)CPU.bus_state]);
#endif

  if (Controller.readALEPin() && (CPU.bus_state == CODE)) {
    // First bus cycle of the instruction to execute. Transition to Execute or EmuEnter as appropriate.
    if (CPU.do_emulation && !CPU.in_emulation) {
      change_state(EmuEnter);
    } else {
      change_state(Execute);
    }
  }
}

void handle_emu_enter_state(uint8_t q) {
  if (!READ_MRDC_PIN) {
    // CPU is reading (MRDC active-low)
    if (CPU.bus_state == CODE) {
      // We are reading a code byte
      if (CPU.v_pc < sizeof EMU_ENTER_PROGRAM) {
        // Feed load program to CPU
        CPU.data_bus = read_program(EMU_ENTER_PROGRAM, sizeof EMU_ENTER_PROGRAM, &CPU.v_pc, CPU.address_latch, CPU.data_width);
        CPU.data_type = QueueDataType::Program;
      } else {
        // Ran out of program, so return NOP.
        CPU.data_bus = OPCODE_DOUBLENOP;
        CPU.data_type = QueueDataType::ProgramEnd;
        //change_state(LoadDone);
      }
      data_bus_write(CPU.data_bus, CPU.data_width);
    }

    if (CPU.bus_state == MEMR) {
      // We are reading from memory
      // This will occur when BRKEM reads the emulation segment vector
      uint32_t vector_base = BRKEM_VECTOR * 4;
      if ((CPU.address_latch >= vector_base) && (CPU.address_latch < vector_base + 4)) {
        if (CPU.address_latch < (vector_base + 2)) {

#if DEBUG_EMU
          // Reading offset, feed IP
          DEBUG_SERIAL.println("## Reading BRKEM offset! ##");
#endif
        } else {
#if DEBUG_EMU
          // Reading segment
          DEBUG_SERIAL.println("## Reading BRKEM segment! ##");
#endif
        }
        // Feed a dummy pc variable to read_program - the actual address is determined from
        // the address latch
        uint16_t dummy_pc = (uint16_t)(CPU.address_latch - vector_base);
        CPU.data_bus = read_program(EMU_ENTER_PROGRAM, sizeof EMU_ENTER_PROGRAM, &dummy_pc, CPU.address_latch, CPU.data_width);
        CPU.data_type = QueueDataType::Program;
        data_bus_write(CPU.data_bus, CPU.data_width);
      } else {
        // Unexpected read above address 0x00001
        DEBUG_SERIAL.println("## INVALID MEM READ DURING EMUENTER ##");
      }
    }
  }

  if (!READ_MWTC_PIN) {
    if (CPU.width == BusWidthEight) {
      // Flags will be read in two operations
      if (CPU.stack_w_op_ct == 0) {
#if DEBUG_EMU
        DEBUG_SERIAL.println("## Reading BRKEM flag push (1/2)! ##");
#endif
        CPU.pre_emu_flags = (uint16_t)data_bus_read_byte();
      } else if (CPU.stack_w_op_ct == 1) {
#if DEBUG_EMU
        DEBUG_SERIAL.println("## Reading BRKEM flag push (2/2)! ##");
#endif
        CPU.pre_emu_flags |= ((uint16_t)data_bus_read_byte() << 8);
      }
      CPU.stack_w_op_ct++;
    } else {
      // Flags will be read in one operation
      if (CPU.stack_w_op_ct == 0) {
#if DEBUG_EMU
        DEBUG_SERIAL.println("## Reading BRKEM flag push! ##");
#endif
        // CPU is writing to the data bus, latch value
        CPU.data_bus = data_bus_read(CPU.data_width);
        CPU.pre_emu_flags = CPU.data_bus;
      }
      CPU.stack_w_op_ct++;
    }
  }

  if (q == QUEUE_FLUSHED) {
    // Queue flush after final jump triggers next state.
    CPU.in_emulation = true;
    change_state(LoadDone);
  }
}

// Reverse the order of the stack buffer.
void reverse_stack_buf() {
  uint16_t temp;
  temp = ((uint16_t *)NMI_STACK_BUFFER)[2];
  ((uint16_t *)NMI_STACK_BUFFER)[2] = ((uint16_t *)NMI_STACK_BUFFER)[0];
  ((uint16_t *)NMI_STACK_BUFFER)[0] = temp;
}

// Returns true if the current cycle is the cycle we should write or read
// the data bus. Before calling this function it is assumed we have ensured
// that we are in a bus cycle and one of the 8288 signals is active.
bool is_transfer_cycle() {
  return (READ_READY_PIN && (CPU.bus_cycle == T3 || CPU.bus_cycle == TW));
}

// Return true if the current m-cycle has finished
bool is_transfer_done() {
  switch (CPU.bus_state_latched) {
    case IOR:
      // IORC is active-low, so we are returning true if it is off
      return READ_IORC_PIN;
    case IOW:
      // IOWC is active-low, so we are returning true if it is off
      return READ_IOWC_PIN;
    case CODE:
      // FALLTHRU
    case MEMR:
      // MRDC is active-low, so we are returning true if it is off
      return READ_MRDC_PIN;
    case MEMW:
      // MWTC is active-low, so we are returning true if it is off
      return READ_MWTC_PIN;
    default:
      // Rely on external READY pin
      return READ_READY_PIN;
      break;
  }
}

void print_addr(unsigned long addr) {
  static char addr_buf[6];
  snprintf(addr_buf, 6, "%05lX", addr);
  DEBUG_SERIAL.println(addr_buf);
}

void reset_screen() {
  #if GIGA_DISPLAY_SHIELD
    // By freak occurence, the display shield and the CPU share a reset pin.
    if (screen) {
      screen->init();
      Board.debugPrintln(DebugType::SETUP, "Resetting Giga display...");
      screen->updateCell(0, 5, screen->makeColor(255, 255, 255), "Reset...");
    }
  #endif
}

// Detect the CPU type based on the number of CPU cycles spent executing the
// CPU ID program.
void detect_cpu_type(uint32_t cpuid_cycles) {
  if (CPU.width == BusWidthEight) {
    if (cpuid_cycles > 5) {
#if DEBUG_ID
      debugPrintlnColor(DEBUG_ID_COLOR, "detect_cpu_type(): Detected NEC V20");
#endif
      CPU.cpu_type = CpuType::necV20;
    } else {
#if DEBUG_ID
      debugPrintlnColor(DEBUG_ID_COLOR, "detect_cpu_type(): Detected i8088");
#endif
      CPU.cpu_type = CpuType::i8088;
    }
  } else {
    if (cpuid_cycles > 5) {
#if DEBUG_ID
      debugPrintlnColor(DEBUG_ID_COLOR, "detect_cpu_type(): Detected NEC V30");
#endif
      CPU.cpu_type = CpuType::necV30;
    } else {
#if DEBUG_ID
      debugPrintlnColor(DEBUG_ID_COLOR, "detect_cpu_type(): Detected i8086");
#endif
      CPU.cpu_type = CpuType::i8086;
    }
  }
}

void detect_fpu_type() {
  CPU.fpu_type = i8087;
}

void do_frame_update() {
  if (!screen_initialized) {
    // Screen not initialized yet, so skip frame update
    return;
  }
  unsigned long current_ms = millis();
  unsigned long delta_ms = current_ms - last_millis;
  
  frame_ms_accumulator += delta_ms;
  second_ms_accumulator += delta_ms;
  if (frame_ms_accumulator >= SCREEN_UPDATE_MS) {
    if (frame_ms_accumulator > SCREEN_UPDATE_MS) {
      frame_ms_accumulator -= SCREEN_UPDATE_MS;
    } else {
      frame_ms_accumulator = 0;
    }
    // Write the address latch.
    char buf[6];
    snprintf(buf, sizeof(buf), "%05lX", CPU.address_latch);
    screen->updateCell(0, 1, screen->makeColor(128, 128, 255), buf);
    screen->updateCell(1, 1, screen->makeColor(128, 128, 255), MACHINE_STATE_STRINGS[CPU.v_state]);
    fps_counter++;
  }

  if (second_ms_accumulator >= 1000) {
    if (second_ms_accumulator > 1000) {
      second_ms_accumulator -= 1000;
    } else {
      second_ms_accumulator = 0;
    }
    // Update the FPS counter
    int row = screen->rows() - 1;
    screen->updateCell(row, 1, screen->makeColor(255, 255, 255), (String(fps_counter) + "fps").c_str());
    fps_counter = 0;
  }

  last_millis = current_ms;
}

/// Read the `len` number of bytes from the serial port into the specified buffer.
bool readParameterBytes(uint8_t *buf, size_t buf_len, size_t len)  {

  Controller.getBoard().debugPrintln(DebugType::PROTO, "## readParameterBytes(): Reading " + String(len) + " parameter bytes...");
  size_t bytes_read = 0;
  unsigned long start_time = millis();
  uint8_t byte = 0;

  while (bytes_read < len) {
    if (INBAND_SERIAL.available() > 0) {
      byte = (uint8_t)INBAND_SERIAL.read();
      buf[bytes_read++] = byte;
      Controller.getBoard().debugPrintln(DebugType::PROTO, "## readParameterBytes(): Read byte " + String(bytes_read) + ": 0x" + String(byte, HEX));
    } else {
      // Check for timeout
      if (millis() - start_time > CMD_TIMEOUT) {
        Controller.getBoard().debugPrintln(DebugType::ERROR, "## readParameterBytes(): Timeout!");
        return false; // Timeout
      }
    }
  }
  Controller.getBoard().debugPrintln(DebugType::PROTO, "## readParameterBytes(): Successfully read " + String(bytes_read) + " bytes.");
  return true; // Successfully read all bytes
}

// Main sketch loop
void loop() {

  if (screen_init_requested && !screen_initialized) {
    Board.debugPrintln(DebugType::SETUP, "Initializing screen...");
    screen->init();
    Board.debugPrintln(DebugType::SETUP, "Screen initialized!");
    last_millis = millis();

    // Draw cpu status.
    size_t idx = static_cast<size_t>(CPU.cpu_type);
    screen->updateCell(0, 0, screen->makeColor(255, 255, 255), CPU_TYPE_STRINGS[idx]);

    screen_initialized = true;
  }

  do_frame_update();

  switch (SERVER.c_state) {

    case WaitingForCommand:
      if (INBAND_SERIAL.available() > 0) {
        uint8_t cmd_byte = INBAND_SERIAL.read();

        // DEBUG_SERIAL.print("Received opcode: 0x");
        // DEBUG_SERIAL.println(cmd_byte, HEX);

        debug_cmd(CMD_STRINGS[cmd_byte], "received!");
        if (cmd_byte >= (uint8_t)CmdInvalid) {
          send_fail();
          break;
        }

        // Valid command, enter ReadingCommand state
        SERVER.cmd = (server_command)cmd_byte;

        if (cmd_byte == 0) {
          // We ignore command byte 0 (null command)
          break;
        } else if (cmd_byte > MAX_CMD) {
          // Cmd is out of range
          debug_proto("Command out of range!");
          break;
        } else if (CMD_INPUTS[cmd_byte] > 0) {
          // This command requires input bytes before it is executed.
          SERVER.cmd = (server_command)cmd_byte;
          SERVER.cmd_byte_n = 0;
          SERVER.c_state = ReadingCommand;
          SERVER.cmd_bytes_expected = CMD_INPUTS[cmd_byte];
          SERVER.cmd_start_time = millis();  // Get start time for timeout calculation
        } else {
          // Command requires no input, execute immediately
          bool result = V_TABLE[cmd_byte - 1]();
          if (result) {
            debug_proto("Command OK!");
            send_ok();
          } else {
            debug_proto("Command FAIL!");
            send_fail();
          }
        }
      }
      break;

    case ReadingCommand:
      // The previously specified command requires parameter bytes, so read them in, or timeout
      if (INBAND_SERIAL.available() > 0) {
        uint8_t cmd_byte = INBAND_SERIAL.read();

        if (SERVER.cmd_byte_n < MAX_COMMAND_BYTES) {
          // Stil have bytes yet to read
          COMMAND_BUFFER[SERVER.cmd_byte_n] = cmd_byte;
          SERVER.cmd_byte_n++;

          if (SERVER.cmd_byte_n == SERVER.cmd_bytes_expected) {
            // We have received enough parameter bytes to execute the in-progress command.
            bool result = V_TABLE[SERVER.cmd - 1]();
            if (result) {
              send_ok();
            } else {
              send_fail();
            }

            // Revert to listening for command
            SERVER.cmd_byte_n = 0;
            SERVER.cmd_bytes_expected = 0;
            SERVER.c_state = WaitingForCommand;
          }
        }
      } else {
        // No bytes received yet, so keep track of how long we've been waiting
        uint32_t now = millis();
        uint32_t elapsed = now - SERVER.cmd_start_time;

        if (elapsed >= CMD_TIMEOUT) {
          // Timed out waiting for parameter bytes. Send failure and revert to listening for command
          SERVER.cmd_byte_n = 0;
          SERVER.cmd_bytes_expected = 0;
          SERVER.c_state = WaitingForCommand;
          debug_proto("Command timeout!");
          send_fail();
        }
      }
      break;

    case ExecutingCommand:
      break;
  }
}
