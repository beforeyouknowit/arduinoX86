// CycleStateLogger.h
#pragma once

#include <cstdlib>
#include <cstddef>
#include <SDRAM.h>

#include <serial_config.h>

struct __attribute__((packed)) CycleState {
  uint32_t address_bus;
  uint16_t data_bus;
  uint8_t  cpu_state;
  uint8_t  cpu_status0;
  uint8_t  bus_control_bits;
  uint8_t  bus_command_bits;
  uint16_t pins;

  public:
    static constexpr uint16_t ALE = 0x0001; // Address Latch Enable
    static constexpr uint16_t BHE = 0x0002; // Bus High Enable
    static constexpr uint16_t READY = 0x0004; // Ready line
    static constexpr uint16_t LOCK = 0x0008; // Lock line
};

// Maximum number of CycleState entries to hold
#if defined(ARDUINO_GIGA)
#define MAX_CYCLE_STATES 8192
#else
#define MAX_CYCLE_STATES 512
#endif

class CycleStateLogger {
public:
  CycleStateLogger()
    : buffer_(nullptr)
    , next_(0)
    , wrapped_(false)
  {
    #if defined(ARDUINO_GIGA)
      buffer_ = static_cast<CycleState*>(SDRAM.malloc(sizeof(CycleState) * MAX_CYCLE_STATES));
    #else
      buffer_ = static_cast<CycleState*>(std::malloc(sizeof(CycleState) * MAX_CYCLE_STATES));
    #endif

    if (!buffer_) {
      DEBUG_SERIAL.println("CycleStateLogger: Memory allocation failed!");
    }
    else {
      DEBUG_SERIAL.println("CycleStateLogger: Log buffer allocated successfully.");
    }
    reset();
  }

  ~CycleStateLogger() {
      std::free(buffer_);
  }

  // Record a new cycle state
  void log(const CycleState& state) {
    if (!enabled_) return; // Ignore if logging is disabled
    buffer_[next_] = state;
    next_ = (next_ + 1) % MAX_CYCLE_STATES;
    if (next_ == 0) wrapped_ = true;
  }

  // Clear all stored entries
  void reset() {
    next_ = 0;
    wrapped_ = false;
  }

  void enable_logging() {
    enabled_ = true;
  }

  void disable_logging() {
    enabled_ = false;
  }

  // Number of entries currently stored
  size_t len() const {
    return wrapped_ ? MAX_CYCLE_STATES : next_;
  }

  void dump_states() {
    // Dump the current log buffer as raw bytes.
    uint32_t count = len();
#if DEBUG_DUMP    
    DEBUG_SERIAL.print("## CycleStateLogger: Dumping ");
    DEBUG_SERIAL.print(count);
    DEBUG_SERIAL.print(" cycles, ");
#endif    
    uint8_t *count_bytes = reinterpret_cast<uint8_t*>(&count);
    // Write the count first as 4 bytes
    INBAND_SERIAL.write(count_bytes, sizeof(count));
    // Next, write the size in bytes to follow
    size_t size = count * sizeof(CycleState);
#if DEBUG_DUMP
    DEBUG_SERIAL.print(size);
    DEBUG_SERIAL.println(" bytes total.");
#endif    
    uint8_t *size_bytes = reinterpret_cast<uint8_t*>(&size);
    INBAND_SERIAL.write(size_bytes, sizeof(size));
    // Finally, write the actual CycleState entries
    INBAND_SERIAL.write(reinterpret_cast<const uint8_t*>(buffer_), len() * sizeof(CycleState));
  }

private:
  CycleState* buffer_;
  size_t      next_;
  bool        wrapped_;
  bool        enabled_ = true; // Enable/disable logging
};