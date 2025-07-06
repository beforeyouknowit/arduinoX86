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
#ifndef _ARDUINO_X86_H
#define _ARDUINO_X86_H

#include <Arduino.h>
#include <stdint.h>

#include <config.h>
#include <serial_config.h>
#include <ansi_color.h>
#include <BusTypes.h>
#include <Hat.h>
#include <gpio_pins.h>
#include <InstructionQueue.h>
#include <InlineProgram.h>
#include <programs.h>

// Code segment to use for load program.
const uint16_t LOAD_SEG = 0xD000;
const uint16_t STORE_SEG = 0xE000;
const uint32_t NMI_ADDR = 0x00008;

// Maximum size of the processor instruction queue. For 8088 == 4, 8086 == 6. 
#define QUEUE_SIZE 6

// CPU width. Eight if an 8088/V20 is detected on reset, Sixteen if an 8086/V30 is detected. 
typedef enum {
  BusWidthEight,
  BusWidthSixteen,
} cpu_width_t;

// Type of CPU. These are either detected or specified by the configured hat.
enum class CpuType : uint8_t {
  Undetected,
  i8088, 
  i8086,
  necV20,
  necV30,
  i80188,
  i80186,
  i80286,
};

typedef enum {
  noFpu,
  i8087,
} fpu_type_t;

// CPU Registers - for new NMI STORE routine
typedef struct __attribute__((packed)) registers1 {
  uint16_t ax;
  uint16_t bx;
  uint16_t cx;
  uint16_t dx;
  uint16_t ip;
  uint16_t cs;
  uint16_t flags;
  uint16_t ss;
  uint16_t sp;
  uint16_t ds;
  uint16_t es;
  uint16_t bp;
  uint16_t si;
  uint16_t di;
} registers1_t;

// CPU Registers - for original STORE routine
typedef struct __attribute__((packed)) registers2 {
  uint16_t ax;
  uint16_t bx;
  uint16_t cx;
  uint16_t dx;
  uint16_t ss;
  uint16_t sp;
  uint16_t flags;
  uint16_t ip;
  uint16_t cs;
  uint16_t ds;
  uint16_t es;
  uint16_t bp;
  uint16_t si;
  uint16_t di;
} registers2_t;


struct __attribute__((packed)) SegmentDescriptor {
  uint16_t addr_lo;
  uint8_t addr_hi;
  uint8_t access;
  uint16_t limit;
};

// CPU Registers - for 286 LOADALL command
struct __attribute__((packed)) Loadall286 {
  uint16_t x0;
  uint16_t x1;
  uint16_t x2;
  uint16_t MSW;
  uint16_t x3;
  uint16_t x4;
  uint16_t x5;
  uint16_t x6;
  uint16_t x7;
  uint16_t x8;
  uint16_t x9;
  uint16_t TR;
  uint16_t FLAGS;
  uint16_t IP;
  uint16_t LDT;
  uint16_t DS;
  uint16_t SS;
  uint16_t CS;
  uint16_t ES;
  uint16_t DI;
  uint16_t SI;
  uint16_t BP;
  uint16_t SP;
  uint16_t BX;
  uint16_t DX;
  uint16_t CX;
  uint16_t AX;
  SegmentDescriptor ES_DESC;
  SegmentDescriptor CS_DESC;
  SegmentDescriptor SS_DESC;
  SegmentDescriptor DS_DESC;
  SegmentDescriptor GDT_DESC;
  SegmentDescriptor LDT_DESC;
  SegmentDescriptor IDT_DESC;
  SegmentDescriptor TSS_DESC;
};

#define LOADALL286_ADDRESS 0x800

// Processor instruction queue
typedef struct queue {
  uint8_t queue[QUEUE_SIZE];
  uint8_t types[QUEUE_SIZE];
  size_t size;
  uint8_t front;
  uint8_t back;
  uint8_t len;
} Queue;

#define QUEUE_IDLE 0x00
#define QUEUE_FIRST 0x01
#define QUEUE_FLUSHED 0x02
#define QUEUE_SUBSEQUENT 0x03

// Strings for pretty-printing instruction queue status from QS0,QS1
// '.' = Idle  
// 'F' = First byte fetched 
// 'E' = Queue Emptied 
// 'S' = Subsequent byte fetched
const char QUEUE_STATUS_CHARS[] = {
  ' ', 'F', 'E', 'S'
};

typedef struct program_stats {
  uint16_t code_read_xfers;
  uint16_t memory_read_xfers;
  uint16_t memory_write_xfers;
  uint16_t io_read_xfers;
  uint16_t io_write_xfers;
  uint32_t idle_cycles;
  uint32_t program_cycles;
} p_stats;

// Main CPU State
typedef struct cpu {
  bool doing_reset;
  bool doing_id;
  CpuType cpu_type; // Detected type of the CPU.
  fpu_type_t fpu_type; // Detected type of FPU (0 if none)
  cpu_width_t width; // Native bus width of the CPU. Detected on reset from BHE line.
  bool do_emulation; // Flag that determines if we enter 8080 emulation mode after Load
  bool in_emulation; // Flag set when we have entered 8080 emulation mode and cleared when we have left
  bool do_prefetch; // Flag that determines if we enter Prefetch state and execute a prefetch program.
  uint32_t cpuid_counter; // Cpuid cycle counter. Used to time to identify the CPU type.
  uint32_t cpuid_queue_reads; // Number of queue reads since reset of Cpuid cycle counter.
  uint32_t state_begin_time;
  uint32_t address_bus;
  uint32_t address_latch;
  BusStatus bus_state_latched; // Bus state latched on T1 and valid for entire bus cycle (immediate bus state goes PASV on T3)
  BusStatus bus_state; // Bus state is current status of S0-S2 at given cycle (may not be valid)
  TCycle last_bus_cycle;
  TCycle bus_cycle;
  ActiveBusWidth data_width; // Current size of data bus. Detected during bus transfer from BHE line.
  uint16_t data_bus;
  bool data_bus_resolved; // Whether we have resolved the data bus this m-cycle or not.
  bool prefetching_store;
  uint8_t reads_during_prefetching_store;
  QueueDataType data_type;
  uint8_t status0; // S0-S5, QS0 & QS1
  uint8_t command_bits; // 8288 command outputs
  uint8_t control_bits; // 8288 control outputs
  uint16_t v_pc; // Virtual program counter
  uint16_t s_pc; // Store program counter
  uint16_t stack_r_op_ct; // Number of stack read operations in current state
  uint16_t stack_w_op_ct; // Number of stack write operations in current state
  uint16_t pre_emu_flags; // Flags pushed to stack by BRKEM
  uint8_t emu_flags; // Flags pushed to stack by PUSH PSW during EmuExit program
  volatile registers1_t load_regs; // Register state set by Load command
  volatile Loadall286 loadall_regs; // Register state set by Loadall command
  volatile registers1_t post_regs; // Register state retrieved from Store program
  uint8_t *readback_p;
  bool have_queue_status; // Whether we have access to the queue status lines. Can be detected during RESET.
  InstructionQueue queue; // Instruction queue
  uint8_t opcode; // Currently executing opcode
  const char *mnemonic; // Decoded mnemonic
  uint8_t qb; // Last byte value read from queue
  QueueDataType qt; // Last data type read from queue
  bool q_ff; // Did we fetch a first instruction byte from the queue this cycle?
  uint8_t q_fn; // What # byte of instruction did we fetch?
  bool nmi_terminate; // Whether we are entering ExecuteFinalize via NMI termination.
  uint8_t nmi_checkpoint; // How many reads we have done at the NMI IVT address.
  uint16_t nmi_buf_cursor;
  InlineProgram *program = &JUMP_VECTOR;
  CallStackFrame nmi_stack_frame; // NMI stack frame for 286/386 CPUs
} Cpu;

typedef struct i8288 {
  BusStatus last_status; // S0-S2 of previous cycle
  BusStatus status; // S0-S2 of current cycle
  BusStatus status_latch;
  TCycle tcycle;
  bool ale;
  bool mrdc;
  bool amwc;
  bool mwtc;
  bool iorc;
  bool aiowc;
  bool iowc;
  bool inta;
} Intel8288;

// ----------------------------- CPU FLAGS ----------------------------------//
const uint16_t CPU_FLAG_CARRY      = 0b0000000000000001;
const uint16_t CPU_FLAG_RESERVED1  = 0b0000000000000010;
const uint16_t CPU_FLAG_PARITY     = 0b0000000000000100;
const uint16_t CPU_FLAG_RESERVED3  = 0b0000000000001000;
const uint16_t CPU_FLAG_AUX_CARRY  = 0b0000000000010000;
const uint16_t CPU_FLAG_RESERVED5  = 0b0000000000100000;
const uint16_t CPU_FLAG_ZERO       = 0b0000000001000000;
const uint16_t CPU_FLAG_SIGN       = 0b0000000010000000;
const uint16_t CPU_FLAG_TRAP       = 0b0000000100000000;
const uint16_t CPU_FLAG_INT_ENABLE = 0b0000001000000000;
const uint16_t CPU_FLAG_DIRECTION  = 0b0000010000000000;
const uint16_t CPU_FLAG_OVERFLOW   = 0b0000100000000000;

#define CPU_FLAG_DEFAULT_SET_8086 0xF002
#define CPU_FLAG_DEFAULT_SET_286 0x0002
#define CPU_FLAG_DEFAULT_CLEAR 0xFFD7
// ----------------------------- GPIO PINS ----------------------------------//

#define SPIN_DELAY(count) do { \
    volatile unsigned int _i; \
    for (_i = 0; _i < (count); _i++) { \
        __asm__ __volatile__("nop"); \
    } \
} while(0)

// Time in microseconds to wait after setting clock HIGH or LOW

#if defined(__SAM3X8E__) // If Arduino DUE
  #define CLOCK_PIN_DELAY (void)0
  #define CLOCK_PIN_HIGH_DELAY SPIN_DELAY(1000)
  #define CLOCK_PIN_LOW_DELAY SPIN_DELAY(1000)
#elif defined(ARDUINO_GIGA) 
  #define CLOCK_PIN_DELAY ((void)0)
  #define CLOCK_PIN_HIGH_DELAY ((void)0)
  #define CLOCK_PIN_LOW_DELAY ((void)0)
#endif

// Microseconds to wait after a pin direction change. Without some sort of delay
// a subsequent read/write may fail. You may need to tweak this if you have a 
// different board - some types need longer delays

#if defined(__AVR_ATmega2560__) // Arduino MEGA
  #if BOARD_TYPE == ELEGOO_MEGA 
    #define PIN_CHANGE_DELAY (delayMicroseconds(3))
  #elif BOARD_TYPE == ARDUINO_MEGA
    #define PIN_CHANGE_DELAY (delayMicroseconds(1))
  #endif
#elif defined(__SAM3X8E__) // If Arduino DUE
  #define PIN_CHANGE_DELAY ((void)0)
#elif defined(ARDUINO_GIGA)
  #define PIN_CHANGE_DELAY ((void)0)
#endif

// Bit reverse LUT from http://graphics.stanford.edu/~seander/bithacks.html#BitReverseTable
static const uint8_t BIT_REVERSE_TABLE[256] = 
{
#   define R2(n)    n,     n + 2*64,     n + 1*64,     n + 3*64
#   define R4(n) R2(n), R2(n + 2*16), R2(n + 1*16), R2(n + 3*16)
#   define R6(n) R4(n), R4(n + 2*4 ), R4(n + 1*4 ), R4(n + 3*4 )
    R6(0), R6(2), R6(1), R6(3)
};

#ifndef OPCODE_NOP
  #define OPCODE_NOP 0x90
#endif

// --------------------- Function declarations --------------------------------

// main.cpp
void cycle();
void set_error(const char *msg);
void clear_error();
bool cpu_id();
void patch_load_pgm(InlineProgram *pgm, volatile registers1_t *reg);
void patch_brkem_pgm(InlineProgram *pgm, volatile registers1_t *regs);

void convert_inline_registers(volatile void *inline_regs);
void reverse_stack_buf();
bool is_transfer_done();
void handle_fetch(uint8_t q);
void handle_cpuid_state(uint8_t q);
void handle_cpu_setup_state();
void handle_jump_vector_state(uint8_t q);
void handle_loadall_286();
void handle_load_state(uint8_t q);
void handle_load_done_state();
void handle_emu_enter_state(uint8_t q);
void detect_fpu_type();
void detect_cpu_type(uint32_t cpuid_cycles);
void reset_screen();
bool readParameterBytes(uint8_t *buf, size_t buf_len, size_t len);

uint32_t calc_flat_address(uint16_t seg, uint16_t offset);

void reset_cpu_struct(bool reset_regs);
void clock_tick();
void data_bus_write(uint16_t data, cpu_width_t width);
uint16_t data_bus_read();

// cpu.cpp
void read_address(bool peek);
void read_status0();
uint8_t read_status0_raw();
bool cpu_reset();
void cpu_set_width(cpu_width_t width);
const char *get_opcode_str(uint8_t op1, uint8_t op2, bool modrm);

// bus.cpp
void data_bus_write(uint16_t data, ActiveBusWidth width);
uint16_t data_bus_read(ActiveBusWidth width);
uint16_t data_bus_peek(cpu_width_t width);
void read_address();
uint32_t peek_address();
void latch_address();
bool a0();
uint32_t read_address_pins(bool peek);

// buzzer.cpp
void beep(uint32_t time);
void error_beep();



#endif // _ARDUINO_X86_H