// Main user config. You may wish to edit things here to suit your board and CPU.

// Define this if you are using the Display Shield for the Arduino Giga.
// Warning: greatly increases compile time.
#define GIGA_DISPLAY_SHIELD 1
#define SCREEN_UPDATE_FPS 30
#define SCREEN_UPDATE_MS (1000 / SCREEN_UPDATE_FPS)

// Define this for either a 186 or a 188, we will detect the bus width.
#define CPU_186 0

// Define this if you have connected an 8087 FPU
#define FPU_8087 1

// Baud rate is ignored for Arduino DUE as it uses native SerialUSB. This is legacy.
// YOU SHOULD BE USING A DUE.
//
// For Arduino MEGA, Arduino-branded MEGAs should use 460800 baud. ELEGOO branded MEGAs 
// can use 1000000. 
// You can test higher values but these are the values I determined to work without errors.
// Actual limits may be board-specific!
//#define BAUD_RATE 460800
#define BAUD_RATE 1000000

// DEBUG_BAUD_RATE controls the Serial1 speed. Check the documentation of your RS232 interface
// for maximum rated speed. Exceeding it will cause dropped characters or other corruption.
// The popular MAX3232 module has a maximum rate of 250Kbps, so should use a baud rate of 230400.
// The TRS3122E module specified in the BOM can support 1Mbit. I have been using 460800 with it.
#define DEBUG_BAUD_RATE 460800

#define CMD_TIMEOUT 100 // Command timeout in milliseconds
#define MAX_COMMAND_BYTES 28 // Maximum length of command parameter input

// What vector to use for the BRKEM call. No reason to change this really.
#define BRKEM_VECTOR ((uint8_t)0x00)

// Print a character to the debugging output on each load command.
#define LOAD_INDICATOR 1
// Print a character to the debugging output on each store command.
#define STORE_INDICATOR 1

#define RELEASE_MODE 0 // If set, disables all traces and debugs.

#define TRACE_ALL 1 // TRACE_ALL will enable all traces (TRACE_NONE overrides)
#define TRACE_NONE (0 | RELEASE_MODE) // TRACE_NONE will override all set traces

// These defines control tracing and debugging output for each state.
// Note: tracing a STORE operation will likely cause it to timeout on the client.
#define TRACE_RESET     ((1 | TRACE_ALL) & ~TRACE_NONE) // Print cycle traces during CPU Reset.
#define TRACE_SETUP     ((1 | TRACE_ALL) & ~TRACE_NONE) // Print cycle traces for the CpuSetup state.
#define TRACE_VECTOR    ((1 | TRACE_ALL) & ~TRACE_NONE) // Print cycle traces for the JumpVector state.
#define TRACE_LOAD      ((0 | TRACE_ALL) & ~TRACE_NONE) // Print cycle traces for the Load state.
#define TRACE_ID        ((0 | TRACE_ALL) & ~TRACE_NONE) // Print cycle traces for the CpuId state.
#define TRACE_PREFTECH  ((0 | TRACE_ALL) & ~TRACE_NONE) // Print cycle traces for the Prefetch state.
#define TRACE_EMU_ENTER ((0 | TRACE_ALL) & ~TRACE_NONE)
#define TRACE_EMU_EXIT  ((0 | TRACE_ALL) & ~TRACE_NONE)
#define TRACE_EXECUTE   ((1 | TRACE_ALL) & ~TRACE_NONE)
#define TRACE_STORE     ((1 | TRACE_ALL) & ~TRACE_NONE)
#define TRACE_FINALIZE  ((0 | TRACE_ALL) & ~TRACE_NONE)

#define DEBUG_ALL 0  // DEBUG_ALL will enable all debugs (DEBUG_NONE overrides)
#define DEBUG_NONE (0 | RELEASE_MODE) // DEBUG_NONE will override all set debugs

#define DEBUG_STATE     ((1 | DEBUG_ALL) & ~DEBUG_NONE) // Report state changes and time spent in each state
#define DEBUG_RESET     ((1 | DEBUG_ALL) & ~DEBUG_NONE) // Print info about the reset process
#define DEBUG_SETUP     ((1 | DEBUG_ALL) & ~DEBUG_NONE) // Print info about the CPU setup routine, if applicable
#define DEBUG_VECTOR    ((1 | DEBUG_ALL) & ~DEBUG_NONE) // Print info about jump vector program execution
#define DEBUG_ID        ((1 | DEBUG_ALL) & ~DEBUG_NONE) // Print info about CPU identification
#define DEBUG_LOAD      ((1 | DEBUG_ALL) & ~DEBUG_NONE)
#define DEBUG_LOAD_DONE ((0 | DEBUG_ALL) & ~DEBUG_NONE)
#define DEBUG_EXECUTE   ((1 | DEBUG_ALL) & ~DEBUG_NONE)
#define DEBUG_STORE     ((1 | DEBUG_ALL) & ~DEBUG_NONE)
#define DEBUG_FINALIZE  ((1 | DEBUG_ALL) & ~DEBUG_NONE)
#define DEBUG_INSTR     ((0 | DEBUG_ALL) & ~DEBUG_NONE) // Print instruction mnemonics as they are executed from queue
#define DEBUG_EMU       ((0 | DEBUG_ALL) & ~DEBUG_NONE) // Print debugging information concerning 8080 emulation mode state
#define DEBUG_QUEUE     ((0 | DEBUG_ALL) & ~DEBUG_NONE) // Debugging output for queue operations (flushes, regular queue ops are always reported)
#define DEBUG_TSTATE    ((0 | DEBUG_ALL) & ~DEBUG_NONE) // Info about t-state changes (mostly T3/Tw->T4)
#define DEBUG_PIN_CMD   ((1 | DEBUG_ALL) & ~DEBUG_NONE) // Info about pin write commands
#define DEBUG_BUS       ((1 | DEBUG_ALL) & ~DEBUG_NONE) // Info about bus parameters (Width, etc), writes (cmd_write_data_bus)
#define DEBUG_PROTO 0 // Insert debugging messages into serial output (Escaped by ##...##)
#define DEBUG_CMD 0

#define DEBUG_BUS_COLOR (ansi::bright_green)
#define DEBUG_QUEUE_COLOR (ansi::bright_yellow)
#define DEBUG_STORE_COLOR (ansi::magenta)
#define DEBUG_VECTOR_COLOR (ansi::cyan)
#define DEBUG_ID_COLOR (ansi::green)
#define ERROR_COLOR (ansi::bright_red)

#define MAX_ERR_LEN 50 // Maximum length of an error string

#define FINALIZE_TIMEOUT 30
#define FINALIZE_EMU_TIMEOUT 90 // We need more time to exit emulation mode
#define STORE_TIMEOUT 300

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
