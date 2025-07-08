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

#if defined(ARDUINO_GIGA)
#define MEMORY_SIZE (4 * 1024 * 1024) // 4MB for Giga
#else
#define MEMORY_SIZE (0x10000) // 64KB for other boards
#endif

// Maximum number of bus operations to record
static const size_t BUS_LOGGER_MAX_OPS = 256;

// Structure representing a single bus operation
struct BusOperation {
  BusOperationType op_type;
  uint32_t address;
  uint16_t data;
};

class BusLogger {
public:
  void log(const BusOperation& op) {
    if (!enabled_) return; // Ignore if logging is disabled
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
      return frame; // Return empty frame
    }
    frame.ip = ops_[idx].data; // Assuming data holds flags
    frame.cs = ops_[idx - 1].data; // Assuming previous data holds CS
    frame.flags = ops_[idx - 2].data; // Assuming two previous data holds IP
    return frame;
  }
  
  // Indicates whether buffer has wrapped at least once
  bool overflowed() const { return overflow_; }
  // Number of valid entries (up to BUS_LOGGER_MAX_OPS)
  size_t count() const { return count_; }
  void enable() { enabled_ = true; }
  void disable() { enabled_ = false; }
  bool is_enabled() const { return enabled_; }
  void reset() {
    count_ = 0;
    index_ = 0;
    overflow_ = false;
    enabled_ = false;
  }

  const BusOperation* data() const { return ops_; }

private:
  BusOperation  ops_[BUS_LOGGER_MAX_OPS];
  size_t count_ = 0;
  size_t index_ = 0;
  bool   overflow_ = false;
  bool   enabled_ = false;
};

// Abstract interface for bus backing implementations
class IBusBackend {
public:
  virtual ~IBusBackend() {}
  virtual uint8_t  read_u8(uint32_t address) = 0;
  virtual uint16_t read_u16(uint32_t address) = 0;
  virtual uint16_t read_bus(uint32_t address, bool bhe) = 0;
  virtual void     write_u8(uint32_t address, uint8_t  value) = 0;
  virtual void     write_u16(uint32_t address, uint16_t value) = 0;
  virtual void     write_bus(uint32_t address, uint16_t value, bool bhe) = 0;
  virtual uint8_t  io_read_u8(uint16_t port) = 0;
  virtual uint16_t io_read_u16(uint16_t port) = 0;
  virtual void     io_write_u8(uint16_t port, uint8_t  value) = 0;
  virtual void     io_write_u16(uint16_t port, uint16_t value) = 0;
  virtual void     set_memory(uint32_t address, const uint8_t* buffer, size_t length) = 0;
  virtual void     randomize_memory() = 0;
};

class BusEmulator {
public:
  explicit BusEmulator(IBusBackend* backend)
    : backend_(backend) {}

  // Memory reads: isFetch==true logs as CodeFetch
  uint8_t mem_read_u8(uint32_t address, bool isFetch) {
    uint8_t val = backend_->read_u8(address);
    logger_.log({isFetch ? BusOperationType::CodeFetch8 : BusOperationType::MemRead8, address, val});
    return val;
  }
  uint16_t mem_read_u16(uint32_t address, bool isFetch) {
    uint16_t val = backend_->read_u16(address);
    logger_.log({isFetch ? BusOperationType::CodeFetch16 : BusOperationType::MemRead16, address, val});
    return val;
  }
  uint16_t mem_read_bus(uint32_t address, bool bhe, bool isFetch) {
    uint16_t val = backend_->read_bus(address, bhe);
    logger_.log({isFetch ? BusOperationType::CodeFetch16 : BusOperationType::MemRead16, address, val});
    return val;
  }
  void mem_write_u8(uint32_t address, uint8_t value) {
    backend_->write_u8(address, value);
    logger_.log({BusOperationType::MemWrite8, address, value});
  }
  void mem_write_u16(uint32_t address, uint16_t value) {
    backend_->write_u16(address, value);
    logger_.log({BusOperationType::MemWrite16, address, value});
  }
  void mem_write_bus(uint32_t address, uint16_t value, bool bhe) {
    backend_->write_bus(address, value, bhe);
    logger_.log({BusOperationType::MemWrite16, address, value});

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
    logger_.log({BusOperationType::IoRead8, port, val});
    return val;
  }
  uint16_t io_read_u16(uint16_t port) {
    uint16_t val = backend_->io_read_u16(port);
    logger_.log({BusOperationType::IoRead16, port, val});
    return val;
  }
  void io_write_u8(uint16_t port, uint8_t value) {
    backend_->io_write_u8(port, value);
    logger_.log({BusOperationType::IoWrite8, port, value});
  }
  void io_write_u16(uint16_t port, uint16_t value) {
    backend_->io_write_u16(port, value);
    logger_.log({BusOperationType::IoWrite16, port, value});
  }

  void halt(uint32_t address) {
    if ((address & 0x2) != 0) {
      logger_.log({BusOperationType::Halt, address, 0});
    }
    else {
      logger_.log({BusOperationType::Shutdown, address, 0});
    }
  }

  void set_memory(uint32_t address, const uint8_t* buffer, size_t length) {
    backend_->set_memory(address, buffer, length);
  }

  /// @brief Randomizes the contents of the emulated memory with random data.
  void randomize_memory() {
    backend_->randomize_memory();
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

  // Expose log info
  const BusOperation* log_data() const { return logger_.data(); }
  size_t log_count() const { return logger_.count(); }
  bool log_overflowed() const { return logger_.overflowed(); }
  BusOperation log_peek_back(size_t rel) const { return logger_.peek_back(rel); }
  CallStackFrame log_peek_call_frame() const { return logger_.peek_call_frame(); }

  Loadall286& loadall286_regs() {
    return loadall286_regs_;
  }

  ~BusEmulator() {
      delete backend_;
  }

private:
  IBusBackend* backend_;
  BusLogger   logger_;

  // Keep a shadow of Loadall286 registers.
  Loadall286 loadall286_regs_;
};


class SdramBackend : public IBusBackend {
public:
  SdramBackend(size_t size, size_t mask)
    : size_(size), mask_(mask) {
      SDRAM.begin();
      mem_ = (uint8_t*)SDRAM.malloc(4 * 1024 * 1024);
      if (!mem_) {
          DEBUG_SERIAL.println("## SDRAM: Failed to allocate memory!");
          size_ = 0;
      }
      else {
          DEBUG_SERIAL.print("## SDRAM: Allocated ");
          DEBUG_SERIAL.print(size_);
          DEBUG_SERIAL.println(" bytes memory");
      }
    }                       

  uint8_t read_u8(uint32_t addr) override {
    return mem_[addr & mask_];
  };
  uint16_t read_u16(uint32_t addr) override {
    uint32_t masked_addr = addr & mask_;
    return (mem_[masked_addr] | (mem_[masked_addr + 1] << 8));
  };
  uint16_t read_bus(uint32_t addr, bool bhe) override {
    bool a0 = (addr & 1);
    size_t mask16 = mask_ >> 1; // Mask for 16-bit access
    size_t addr16 = addr >> 1; // Convert to 16-bit address
    uint16_t *mem16 = reinterpret_cast<uint16_t*>(mem_);
    uint16_t bus_val = 0;
    if (a0 && bhe) {
        // Return addr in high byte
        return (mem16[addr16 & mask16] << 8);
    } 
    else if (!a0 && bhe) {
        // Return full 16-bit value
        bus_val = mem16[addr16 & mask16];
        bus_val |= static_cast<uint16_t>(mem16[(addr16 + 1) & mask16]) << 8;
        return bus_val;
    }
    else {
        // Return low byte only
        return mem16[addr16 & mask16];
    }
  };
  void write_u8(uint32_t addr, uint8_t val) override {
    mem_[addr & mask_] = val;
  };
  void write_u16(uint32_t addr, uint16_t val) override {
      uint32_t masked_addr0 = addr & mask_;
      uint32_t masked_addr1 = (addr + 1) & mask_;
    mem_[masked_addr0] = (uint8_t)(val & 0xFF);
    mem_[masked_addr1] = (uint8_t)(val >> 8);
  };
  void write_bus(uint32_t addr, uint16_t val, bool bhe) override {
    if (!mem_) {
      return;
    }
    bool a0 = (addr & 1);
    size_t mask16 = mask_ >> 1; // Mask for 16-bit access
    size_t addr16 = addr >> 1; // Convert to 16-bit address
    uint16_t *mem16 = reinterpret_cast<uint16_t*>(mem_);
    if (a0 && bhe) {
        // Write high byte only
        mem16[addr16 & mask16] = (val >> 8) & 0xFF;
    } 
    else if (!a0 && bhe) {
        // Write full 16-bit value
        mem16[addr16 & mask16] = (val & 0xFF);
        mem16[(addr16 + 1) & mask16] = (val >> 8) & 0xFF;
    }
    else {
        // Write low byte only
        mem16[addr16 & mask16] = (val & 0xFF);
    }
  };

  uint8_t io_read_u8(uint16_t port) override {
    return 0xFF;
  };
  uint16_t io_read_u16(uint16_t port) override {
    return 0xFFFF;
  };
  void io_write_u8(uint16_t port, uint8_t val) override {
    return;
  };
  void io_write_u16(uint16_t port, uint16_t val) override {
    return;
  };

  void set_memory(uint32_t address, const uint8_t* buffer, size_t length) override {
    if (address + length > size_) {
        DEBUG_SERIAL.println("## SDRAM: Attempt to write beyond SDRAM bounds");
        return;
    }
    memcpy(mem_ + (address & mask_), buffer, length);
  };

  void randomize_memory() override {
    if (!mem_) {
      return;
    }
    // Fill memory with random data, 32-bits at a time for speed.
    uint32_t *fast_ptr = reinterpret_cast<uint32_t*>(mem_);

    for (size_t i = 0; i < (size_ / 4); i++) {
      uint32_t random_u32 = static_cast<uint32_t>(random(UINT32_MAX));
      fast_ptr[i] = random_u32;
    }
  };

private:
  size_t   size_;
  size_t   mask_;
  uint8_t* mem_;
};

// Null backend: does nothing and returns zero
class NullBackend : public IBusBackend {
public:
  uint8_t  read_u8(uint32_t) override { return 0; }
  uint16_t read_u16(uint32_t) override { return 0; }
  uint16_t read_bus(uint32_t, bool) override { return 0; }
  void     write_u8(uint32_t, uint8_t) override {}
  void     write_u16(uint32_t, uint16_t) override {}
  void     write_bus(uint32_t, uint16_t, bool) override {}
  uint8_t  io_read_u8(uint16_t) override { return 0; }
  uint16_t io_read_u16(uint16_t) override { return 0; }
  void     io_write_u8(uint16_t, uint8_t) override {}
  void     io_write_u16(uint16_t, uint16_t) override {}
  void     set_memory(uint32_t, const uint8_t*, size_t) override {}
  void     randomize_memory() override {}
};

// Factory helper: choose backend based on platform
inline BusEmulator* create_bus_emulator() {
#ifdef ARDUINO_GIGA
  return new BusEmulator(
      new SdramBackend(MEMORY_SIZE, ADDRESS_SPACE_MASK)
  );
#else
  return new BusEmulator(
      new NullBackend()
  );
#endif
}
