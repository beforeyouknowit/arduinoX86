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

#include "hats/Hat8088.h"

#include "cpu_server.h"
#include "gpio_pins.h"

// Nothing in here should need modification. User parameters can be set in cpu_server.h

#if CPU_186
  // 186 CPU
  // How many cycles to assert the RESET pin.
  #define RESET_HOLD_CYCLE_COUNT 30
  // How many cycles it takes to reset the CPU after RESET signal de-asserts. First ALE should occur after this many cycles.
  #define RESET_CYCLE_COUNT 35
  // If we didn't see an ALE after this many cycles, give up
  #define RESET_CYCLE_TIMEOUT 45
  // What logic level RESET is when asserted
  #define RESET_ASSERT 0
  // What logic level RESET is when deasserted
  #define RESET_DEASSERT 1
  // The 186 doesn't need an 8288. We can synthesize 8288 outputs using the CPU's own RD & WR & S2 signals.
  // Leave this value at 0 when using a 186.
  #define EMULATE_8288 0
  // If you are using a newer 186 like an 80L186EB it won't have queue status lines.
  // Set this to 0 in that case to use alternate logic.
  #define HAVE_QUEUE_STATUS 0
  // 80186 needs setup to enable interrupts. 
  #define USE_SETUP_PROGRAM 1
  #define SETUP_PROGRAM SETUP_PROGRAM_186
  #define SETUP_PROGRAM_PATCH_OFFSET SETUP_PATCH_VECTOR_OFFSET_186
#else
  // Non-186 CPU
  // How many cycles to hold the RESET signal high. Intel says "greater than 4" although 4 seems to work.
  #define RESET_HOLD_CYCLE_COUNT 5
  // How many cycles it takes to reset the CPU after RESET signal goes low. First ALE should occur after this many cycles.
  #define RESET_CYCLE_COUNT 7
  // If we didn't see an ALE after this many cycles, give up
  #define RESET_CYCLE_TIMEOUT 20
  // What logic level RESET is when asserted
  #define RESET_ASSERT 1
  // What logic level RESET is when deasserted
  #define RESET_DEASSERT 0
  // Set this to 1 to use i8288 emulation
  #define EMULATE_8288 1
  // Leave this at 1 for non-186 CPUs as they will always have the queue status lines.
  #define HAVE_QUEUE_STATUS 1
  // 8086 needs no setup. Disable entering CpuSetup state.
  #define USE_SETUP_PROGRAM 0
  #define SETUP_PROGRAM SETUP_PROGRAM_86
  #define SETUP_PROGRAM_PATCH_OFFSET 0
#endif

#include "hat_config.h"
#include "ansi_color.h"

// Define board type 
#define ARDUINO_MEGA 1
#define ARDUINO_DUE 2
#define ARDUINO_GIGA 3

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

// Data bus width. There are three possible data bus states:
// - the low 8 bits are active,
// - the high 8 bits are active,
// - all 16 bits are active
typedef enum {
  EightLow,
  EightHigh,
  Sixteen,
} data_width_t;

// CPU type. Arduino8088 attempts to detect these. These are aliased to the byte values 0-5.
typedef enum {
  i8088, 
  i8086,
  necV20,
  necV30,
  i80188,
  i80186,
} cpu_type_t;

const char *CPU_TYPE_STRINGS[] = {
  "i8088",
  "i8086",
  "NEC V20",
  "NEC V30",
  "i80188",
  "i80186"
};


typedef enum {
  noFpu,
  i8087,
} fpu_type_t;

const char *FPU_TYPE_STRINGS[] = {
  "None",
  "i8087",
};


const char CPU_TYPE_COUNT = sizeof(CPU_TYPE_STRINGS) / sizeof(CPU_TYPE_STRINGS[0]);

// Bus transfer states, as determined by status lines S0-S2.
typedef enum {
  IRQA = 0,   // IRQ Acknowledge
  IOR  = 1,   // IO Read
  IOW  = 2,   // IO Write
  HALT = 3,   // Halt
  CODE = 4,   // Code
  MEMR = 5,   // Memory Read
  MEMW = 6,   // Memory Write
  PASV = 7    // Passive
} s_state;

// Strings for printing bus states cycles.
const char *BUS_STATE_STRINGS[] = {
  "IRQA",
  "IOR ",
  "IOW ",
  "HALT",
  "CODE",
  "MEMR",
  "MEMW",
  "PASV"
};

const char *BUS_STATE_COLORS[] = {
  ansi::bright_red,     // IRQA 
  ansi::yellow,         // IOR  
  ansi::bright_yellow,  // IOW  
  ansi::bright_magenta, // HALT 
  ansi::cyan,           // CODE 
  ansi::bright_blue,    // MEMR 
  ansi::bright_green,   // MEMW 
  ansi::white           // PASV 
};

// Bus transfer cycles. Tw is wait state, inserted if READY is not asserted during T3.
typedef enum { 
  T1 = 0,
  T2 = 1,
  T3 = 2,
  T4 = 3,
  TW = 4,
  TI = 5,
} t_cycle_t;


// Strings for printing bus transfer cycles.
const char *CYCLE_STRINGS[] = {
  "T1", "T2", "T3", "T4", "Tw", "Ti"
};

const char *SEGMENT_STRINGS[] = {
  "ES", "SS", "CS", "DS"
};

// CPU Registers - for new NMI STORE routine
typedef struct registers1 {
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
} registers1_t __attribute__((packed));

// CPU Registers - for original STORE routine
typedef struct registers2 {
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
} registers2_t __attribute__((packed));

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

// Data bus data types. These are stored when pushing to the prefetch queue, so we know what 
// kind of byte we are retrieving from the processor queue. This allows us to detect program
// end when the first non-program byte is fetched as the first byte of an instruction.
#define DATA_PROGRAM 0x00
#define DATA_PROGRAM_END 0x01

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
  cpu_type_t cpu_type; // Detected type of the CPU.
  fpu_type_t fpu_type; // Detected type of FPU (0 if none)
  cpu_width_t width; // Native bus width of the CPU. Detected on reset from BHE line.
  bool do_emulation; // Flag that determines if we enter 8080 emulation mode after Load
  bool in_emulation; // Flag set when we have entered 8080 emulation mode and cleared when we have left
  bool do_prefetch; // Flag that determines if we enter Prefetch state and execute a prefetch program.
  uint32_t cpuid_counter; // Cpuid cycle counter. Used to time to identify the CPU type.
  uint32_t cpuid_queue_reads; // Number of queue reads since reset of Cpuid cycle counter.
  machine_state_t v_state;
  uint32_t state_begin_time;
  uint32_t address_bus;
  uint32_t address_latch;
  s_state bus_state_latched; // Bus state latched on T1 and valid for entire bus cycle (immediate bus state goes PASV on T3)
  s_state bus_state; // Bus state is current status of S0-S2 at given cycle (may not be valid)
  t_cycle_t bus_cycle;
  data_width_t data_width; // Current size of data bus. Detected during bus transfer from BHE line.
  uint16_t data_bus;
  bool data_bus_resolved; // Whether we have resolved the data bus this m-cycle or not.
  bool prefetching_store;
  uint8_t reads_during_prefetching_store;
  uint8_t data_type;
  uint8_t status0; // S0-S5, QS0 & QS1
  uint8_t command_bits; // 8288 command outputs
  uint8_t control_bits; // 8288 control outputs
  uint16_t v_pc; // Virtual program counter
  uint16_t s_pc; // Store program counter
  uint16_t stack_r_op_ct; // Number of stack read operations in current state
  uint16_t stack_w_op_ct; // Number of stack write operations in current state
  uint16_t pre_emu_flags; // Flags pushed to stack by BRKEM
  uint8_t emu_flags; // Flags pushed to stack by PUSH PSW during EmuExit program
  registers1_t load_regs; // Register state set by Load command
  volatile registers1_t post_regs; // Register state retrieved from Store program
  uint8_t *readback_p;
  bool have_queue_status; // Whether we have access to the queue status lines. Can be detected during RESET.
  Queue queue; // Instruction queue
  uint8_t opcode; // Currently executing opcode
  const char *mnemonic; // Decoded mnemonic
  uint8_t qb; // Last byte value read from queue
  uint8_t qt; // Last data type read from queue
  bool q_ff; // Did we fetch a first instruction byte from the queue this cycle?
  uint8_t q_fn; // What # byte of instruction did we fetch?
  bool nmi_terminate; // Whether we are entering ExecuteFinalize via NMI termination.
  uint8_t nmi_checkpoint; // How many reads we have done at the NMI IVT address.
  uint16_t nmi_buf_cursor;

  const uint8_t *program;
  size_t program_len;
  uint16_t *program_pc;
} Cpu;

typedef struct i8288 {
  s_state last_status; // S0-S2 of previous cycle
  s_state status; // S0-S2 of current cycle
  s_state status_latch;
  t_cycle_t tcycle;
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

#define CPU_FLAG_DEFAULT_SET 0xF002
#define CPU_FLAG_DEFAULT_CLEAR 0xFFD7
// ----------------------------- GPIO PINS ----------------------------------//

#define SPIN_DELAY(count) do { \
    volatile unsigned int _i; \
    for (_i = 0; _i < (count); _i++) { \
        __asm__ __volatile__("nop"); \
    } \
} while(0)

// Time in microseconds to wait after setting clock HIGH or LOW

#if defined(__AVR_ATmega2560__) // Arduino MEGA
  #define CLOCK_PIN_DELAY 0
  #define CLOCK_PIN_HIGH_DELAY (delayMicroseconds(0))
  #define CLOCK_PIN_LOW_DELAY (delayMicroseconds(0))

#elif defined(__SAM3X8E__) // If Arduino DUE
  #define CLOCK_PIN_DELAY 0
  #define CLOCK_PIN_HIGH_DELAY SPIN_DELAY(1000)
  #define CLOCK_PIN_LOW_DELAY SPIN_DELAY(1000)

#elif defined(ARDUINO_GIGA) 
  #define CLOCK_PIN_DELAY 0
  #define CLOCK_PIN_HIGH_DELAY 0
  #define CLOCK_PIN_LOW_DELAY 0

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
  #define PIN_CHANGE_DELAY 0
#elif defined(ARDUINO_GIGA)
  #define PIN_CHANGE_DELAY 0
#endif


// High word of cycle count
unsigned long CYCLE_NUM_H = 0;
// Low word of cycle count
unsigned long CYCLE_NUM = 0;

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
uint32_t calc_flat_address(uint16_t seg, uint16_t offset);

void reset_cpu_struct(bool reset_regs);
void clock_tick();
void data_bus_write(uint16_t data, cpu_width_t width);
uint16_t data_bus_read();

void latch_address();
void read_address(bool peek);
uint32_t peek_address();
void read_status0();
bool cpu_reset();
void cpu_set_width(cpu_width_t width);

void init_queue();
void push_queue(uint16_t data, uint8_t dtype, bool a0);
bool pop_queue(uint8_t *byte, uint8_t *dtype);
void empty_queue();
void print_queue();
void read_queue();

// i8288.ino
void tick_i8288();
void reset_i8288();

// piq.ino
void init_queue();
void push_queue(uint16_t data, uint8_t dtype, data_width_t width);
bool pop_queue(uint8_t *byte, uint8_t *dtype);
bool queue_has_room(data_width_t width);
void empty_queue();
void print_queue();
uint8_t read_queue(size_t idx);
const char *queue_to_string();

// bus.ino
void data_bus_write(uint16_t data, data_width_t width);
uint16_t data_bus_read(data_width_t width);
uint16_t data_bus_peek(cpu_width_t width);
void read_address();
uint32_t peek_address();
void latch_address();
bool a0();
uint32_t read_address_pins(bool peek);

// buzzer.ino
void beep(uint32_t time);
void error_beep();

#endif // _ARDUINO_X86_H