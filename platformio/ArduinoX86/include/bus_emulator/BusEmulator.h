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
#include <Arduino.h>
#include <SDRAM.h>

#include <memory>
#include <cstdint>

#include <Hat.h>
#include <BusTypes.h>
#include <serial_config.h>
#include <arduinoX86.h>
#include <globals.h>

#include <bus_emulator/IBusBackend.h>
#include <bus_emulator/SdramBackend.h>
#include <bus_emulator/HashBackend.h>
#include <bus_emulator/NullBackend.h>

#if defined(ARDUINO_GIGA)
#define MEMORY_SIZE (2 * 1024 * 1024) // 4MB for Giga
#else
#define MEMORY_SIZE (0x10000) // 64KB for other boards
#endif

// Maximum number of bus operations to record
static const size_t BUS_LOGGER_MAX_OPS = 256;

// Structure representing a single bus operation
struct BusOperation {
  BusOperationType op_type;
  ActiveBusWidth bus_width;
  uint32_t address;
  uint16_t data;
};

class BusLogger {
public:
  void log(const BusOperation& op) {
    if (!enabled_) return; // Ignore if logging is disabled
    
    if ((op.op_type == BusOperationType::MemWrite8) ||
       (op.op_type == BusOperationType::MemWrite16)) 
    {
      consecutive_writes_++;
      if (consecutive_writes_ == 3) {
        //DEBUG_SERIAL.println("## BusLogger: Detected 3 consecutive writes. Possible far call or exception.");
      }
    }
    else {
      consecutive_writes_ = 0; // Reset on non-write operations
    }
    ops_[index_] = op;
    index_ = (index_ + 1) % BUS_LOGGER_MAX_OPS;
    if (count_ < BUS_LOGGER_MAX_OPS) {
        ++count_;
    } else {
        overflow_ = true;
    }
  }

  // Returns the most recent entry when relative=0, previous when relative=1, etc.
  // If relative >= count(), behavior is undefined.
  BusOperation peek_back(size_t relative) const {
    size_t idx = (index_ + BUS_LOGGER_MAX_OPS - 1 - relative) % BUS_LOGGER_MAX_OPS;
    return ops_[idx];
  }

  CallStackFrame peek_call_frame() const {
    size_t idx = (index_ + BUS_LOGGER_MAX_OPS - 1) % BUS_LOGGER_MAX_OPS;    
    CallStackFrame frame = { 0 };
    if (idx < 2) {
      // Not enough data to form a valid frame
      DEBUG_SERIAL.println("## BusLogger: Not enough data to form a valid call frame!");
      return frame; // Return empty frame
    }

    if (ops_[idx].bus_width == ActiveBusWidth::Sixteen) {
      //DEBUG_SERIAL.println("## BusLogger: Using 16-bit bus width for call frame");
      frame.ip = ops_[idx].data; // Assuming data holds flags
      frame.cs = ops_[idx - 1].data; // Assuming previous data holds CS
      frame.flags = ops_[idx - 2].data; // Assuming two previous data holds IP
      return frame;
    } 
    else {
      //DEBUG_SERIAL.println("## BusLogger: Using 8-bit bus width for call frame");
      // Eight-bit bus width, we need to combine two entries
      if (idx < 5) {
        // Not enough data to form a valid frame
        DEBUG_SERIAL.println("## BusLogger: Not enough data to form a valid call frame!");
        return frame; // Return empty frame
      }
      frame.ip = ((ops_[idx].data & 0x00FF) << 8) | ((ops_[idx - 1].data & 0xFF00) >> 8); // Combine two 8-bit reads for IP
      frame.cs = ((ops_[idx - 2].data & 0x00FF) << 8) | ((ops_[idx - 3].data & 0xFF00) >> 8); // Combine two 8-bit reads for CS
      frame.flags = ((ops_[idx - 4].data & 0x00FF) << 8) | ((ops_[idx - 5].data & 0xFF00) >> 8); // Combine two 8-bit reads for flags
      return frame;
    }
  }
  
  // Indicates whether buffer has wrapped at least once
  bool overflowed() const { return overflow_; }
  // Number of valid entries (up to BUS_LOGGER_MAX_OPS)
  size_t count() const { return count_; }
  void enable() { enabled_ = true; }
  void disable() { enabled_ = false; }
  bool is_enabled() const { return enabled_; }
  int get_consecutive_writes() { return consecutive_writes_; }
  void reset() {
    count_ = 0;
    index_ = 0;
    overflow_ = false;
    enabled_ = false;
    consecutive_writes_ = 0;
  }

  const BusOperation* data() const { return ops_; }

private:
  BusOperation  ops_[BUS_LOGGER_MAX_OPS];
  size_t count_ = 0;
  size_t index_ = 0;
  bool   overflow_ = false;
  bool   enabled_ = false;
  int    consecutive_writes_ = 0; // For detecting far calls/exceptions
};

class BusEmulator {
public:
  explicit BusEmulator(IBusBackend* backend)
    : backend_(backend) {}

  void set_cpu_type(CpuType cpu_type) {
    cpu_type_ = cpu_type;
  }

  // Memory reads: isFetch==true logs as CodeFetch
  uint8_t mem_read_u8(uint32_t address, bool isFetch) {
    uint8_t val = backend_->read_u8(address);
    //logger_.log({isFetch ? BusOperationType::CodeFetch8 : BusOperationType::MemRead8, ActiveBusWidth::E address, val});
    return val;
  }
  uint16_t mem_read_u16(uint32_t address, bool isFetch) {
    uint16_t val = backend_->read_u16(address);
    //logger_.log({isFetch ? BusOperationType::CodeFetch16 : BusOperationType::MemRead16, address, val});
    return val;
  }
  uint16_t mem_read_bus(uint32_t address, bool bhe, bool isFetch) {
    uint16_t val = backend_->read_bus(address, bhe);
    logger_.log({
      isFetch ? BusOperationType::CodeFetch16 : BusOperationType::MemRead16, 
      bus_width(address, bhe), 
      address, 
      val
    });
    return val;
  }
  void mem_write_u8(uint32_t address, uint8_t value) {
    backend_->write_u8(address, value);
    //logger_.log({BusOperationType::MemWrite8, address, value});
  }
  void mem_write_u16(uint32_t address, uint16_t value) {
    backend_->write_u16(address, value);
    //logger_.log({BusOperationType::MemWrite16, address, value});
  }
  void mem_write_bus(uint32_t address, uint16_t value, bool bhe) {
    backend_->write_bus(address, value, bhe);
    logger_.log({BusOperationType::MemWrite16, bus_width(address, bhe), address, value});

    // Write to loadall286 registers if address matches
    if ((address >= 0x800) && (address < (0x800 + sizeof(Loadall286) - 1))) {
      size_t offset = address - 0x800;
      if (offset < sizeof(Loadall286)) {
        uint16_t* reg_ptr = reinterpret_cast<uint16_t*>(&loadall286_regs_) + (offset / 2);
        *reg_ptr = value;
      }
    }
  }

  uint8_t io_read_u8(uint16_t port) {
    uint8_t val = backend_->io_read_u8(port);
    //logger_.log({BusOperationType::IoRead8, port, val});
    return val;
  }
  uint16_t io_read_u16(uint16_t port) {
    uint16_t val = backend_->io_read_u16(port);
    //logger_.log({BusOperationType::IoRead16, port, val});
    return val;
  }
  uint16_t io_read_bus(uint16_t port, bool bhe) {
    uint16_t val = backend_->io_read_bus(port, bhe);
    logger_.log({BusOperationType::IoRead16, bus_width(port, bhe), port, val});
    return val;
  }
  void io_write_u8(uint16_t port, uint8_t value) {
    backend_->io_write_u8(port, value);
    //logger_.log({BusOperationType::IoWrite8, port, value});
  }
  void io_write_u16(uint16_t port, uint16_t value) {
    backend_->io_write_u16(port, value);
    //logger_.log({BusOperationType::IoWrite16, port, value});
  }
  void io_write_bus(uint16_t port, uint16_t value, bool bhe) {
    backend_->io_write_bus(port, value, bhe);
    logger_.log({BusOperationType::IoWrite16, bus_width(port, bhe), port, value});
  
    // Write to Loadall386 registers if port matches
    if (cpu_type_ == CpuType::i80386 && 
        (port >= STORE_IO_BASE) && (port < (STORE_IO_BASE + sizeof(Loadall386) - 1))) 
    {
      size_t offset = port - STORE_IO_BASE;
      if (offset < sizeof(Loadall386)) {
        uint16_t* reg_ptr = reinterpret_cast<uint16_t*>(&loadall386_regs_) + (offset / 2);
        *reg_ptr = value;
      }
    }

  }

  void halt(uint32_t address) {
    if ((address & 0x2) != 0) {
      logger_.log({BusOperationType::Halt,  ActiveBusWidth::Sixteen, address, 0});
    }
    else {
      logger_.log({BusOperationType::Shutdown,  ActiveBusWidth::Sixteen, address, 0});
    }
  }

  void set_memory(uint32_t address, const uint8_t* buffer, size_t length) {
    backend_->set_memory(address, buffer, length);
  }

  void debug_memory(uint32_t address, size_t length) {
    backend_->debug_mem(address, length);
  }

  /// @brief Randomizes the contents of the emulated memory with random data.
  void randomize_memory(uint32_t seed) {
    backend_->randomize_memory(seed);
  }
  void enable_logging() {
    logger_.enable();
  }
  void disable_logging() {
    logger_.disable();
  }
  void reset_logging() {
    logger_.reset();
  }
  bool far_call_detected() {
    // Check that the last 3 bus operations were writes. 
    // This is indicative of a far call or exception if we are reading from the IVT.
    return logger_.get_consecutive_writes() >= 3;
  }
  void set_memory_strategy(IBusBackend::DefaultStrategy strategy, uint32_t start, uint32_t end) {
    backend_->set_strategy(strategy, start, end);
  }

  // Expose log info
  const BusOperation* log_data() const { return logger_.data(); }
  size_t log_count() const { return logger_.count(); }
  bool log_overflowed() const { return logger_.overflowed(); }
  BusOperation log_peek_back(size_t rel) const { return logger_.peek_back(rel); }
  CallStackFrame log_peek_call_frame() const { return logger_.peek_call_frame(); }

  Loadall286& loadall286_regs() {
    return loadall286_regs_;
  }

  Loadall386& loadall386_regs() {
    return loadall386_regs_;
  }

  ~BusEmulator() {
      delete backend_;
  }

private:
  IBusBackend* backend_;
  BusLogger   logger_;
  CpuType cpu_type_ = CpuType::Undetected; // Default CPU type

  // Keep a shadow of Loadall registers.
  Loadall286 loadall286_regs_;
  Loadall386 loadall386_regs_;

  /// @brief Determine bus width based on address and BHE
  ActiveBusWidth bus_width(uint32_t address, bool bhe) const {
    if (address & 1) {
      //DEBUG_SERIAL.println("## BusEmulator: bus_width(): 8-bit high");
      return ActiveBusWidth::EightHigh;
    }
    if (bhe) {
      //DEBUG_SERIAL.println("## BusEmulator: bus_width(): 16-bit");
      return ActiveBusWidth::Sixteen;
    }
    //DEBUG_SERIAL.println("## BusEmulator: bus_width(): 8-bit low");
    return ActiveBusWidth::EightLow;
  }
};

// Factory helper: choose backend based on platform
inline BusEmulator* create_bus_emulator() {
#if defined(ARDUINO_GIGA)
  return new BusEmulator(
      //new SdramBackend(MEMORY_SIZE, ADDRESS_SPACE_MASK)
      new HashBackend()
  );
#else
  return new BusEmulator(
      new NullBackend()
  );
#endif
}
