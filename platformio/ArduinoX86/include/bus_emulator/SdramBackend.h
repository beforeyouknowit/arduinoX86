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

#include <limits>
#include <SDRAM.h>

#include <bus_emulator/IBusBackend.h>
#include <serial_config.h>

class SdramBackend : public IBusBackend {
public:
  SdramBackend(size_t size, size_t mask)
    : size_(size), mask_(mask) {
      
      mem_ = (uint8_t*)SDRAM.malloc(size);
      if (!mem_) {
          DEBUG_SERIAL.println("## SDRAM: Failed to allocate memory!");
          size_ = 0;
      }
      else {
          memset(mem_, 0, size_); // Initialize memory to zero
          DEBUG_SERIAL.print("## SDRAM: Allocated ");
          DEBUG_SERIAL.print(size_);
          DEBUG_SERIAL.println(" bytes memory");
      }
    }                       

  ~SdramBackend() {
    if (mem_) {
      SDRAM.free(mem_);
      mem_ = nullptr;
      DEBUG_SERIAL.println("## SDRAM: Memory freed");
    }
  }

  IBusBackendType type() const override {
    return IBusBackendType::Sdram;
  }
  
  size_t size() const override {
    return size_;
  }

  uint8_t read_u8(uint32_t addr) override {
    return mem_[addr & mask_];
  };
  uint16_t read_u16(uint32_t addr) override {
    uint32_t masked_addr = addr & mask_;
    return (mem_[masked_addr] | (mem_[masked_addr + 1] << 8));
  };

  // Read from the bus. The AT bus is a odd/even arrangement where A0 is not used
  // to address memory. Therefore we can address words by shifting the address
  // right by one bit.
  uint16_t read_bus(uint32_t addr, bool bhe) override {
    //bool a0 = (addr & 1);
    size_t mask16 = mask_ >> 1; // Mask for 16-bit access
    size_t addr16 = addr >> 1; // Convert to 16-bit address
    uint16_t *mem16 = reinterpret_cast<uint16_t*>(mem_);
    // if (a0 && bhe) {
    //     // Return addr in high byte
    //     return mem16[addr16 & mask16];
    // } 
    // else if (!a0 && bhe) {
    //     // Return full 16-bit value
    //     return mem16[addr16 & mask16];
    // }
    // else {
    //     // Return low byte only
    //     return mem16[addr16 & mask16];
    // }
    return mem16[addr16 & mask16];
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

  uint8_t *get_ptr(uint32_t addr) override { 
    return mem_ + addr;
  }

  /// Write to the bus. The AT bus is a odd/even arrangement where A0 is not used
  /// to address memory. Therefore we can address words by shifting the address
  /// right by one bit. A combination of BHE and A0 determines how the write is
  /// performed:
  /// - A0=0, BHE=1: Write full 16-bit value
  /// - A0=1, BHE=1: Write high byte only
  /// - A0=0, BHE=0: Write low byte only
  /// - A0=1, BHE=0: Invalid state, ignored (Refresh cycle on 80186)
  void write_bus(uint32_t addr, uint16_t val, bool bhe) override {
    if (!mem_) {
      return;
    }
    bool a0 = (addr & 1);
    uint16_t existing_word;
    size_t mask16 = mask_ >> 1; // Mask for 16-bit access
    size_t addr16 = addr >> 1; // Convert to 16-bit address
    uint16_t *mem16 = reinterpret_cast<uint16_t*>(mem_);
    if (a0 && bhe) {
        // Write high byte only
        existing_word = mem16[addr16 & mask16] & 0x00FF; // Preserve low byte.
        mem16[addr16 & mask16] = existing_word | (val & 0xFF00); // Set high byte
    } 
    else if (!a0 && bhe) {
        // Write full 16-bit value
        mem16[addr16 & mask16] = val;
    }
    else {
        // Write low byte only
       existing_word = mem16[addr16 & mask16] & 0xFF00; // Preserve high byte.
       mem16[addr16 & mask16] = existing_word | (val & 0x00FF); // Set low byte
    }
  };

  uint8_t io_read_u8(uint16_t port) override {
    return 0xFF;
  };
  uint16_t io_read_u16(uint16_t port) override {
    return 0xFFFF;
  };
  uint16_t io_read_bus(uint16_t port, bool bhe) override {
    return 0xFFFF; // Not implemented for SDRAM
  };
  void io_write_u8(uint16_t port, uint8_t val) override {
    return;
  };
  void io_write_u16(uint16_t port, uint16_t val) override {
    return;
  };
  void io_write_bus(uint16_t port, uint16_t val, bool bhe) override {
    return; // Not implemented for SDRAM
  };

  void set_memory(uint32_t address, const uint8_t* buffer, size_t length) override {
    if (address + length > size_) {
        DEBUG_SERIAL.println("## SDRAM: Attempt to write beyond SDRAM bounds");
        return;
    }
    memcpy(mem_ + (address & mask_), buffer, length);
  };

  void debug_mem(uint32_t address, size_t length) {
    char buf[64];
    if (!mem_) {
      DEBUG_SERIAL.println("## SDRAM: Memory not initialized");
      return;
    }
    if (address + length > size_) {
      DEBUG_SERIAL.println("## SDRAM: Attempt to read beyond SDRAM bounds");
      return;
    }
    for (size_t i = 0; i < length; i++) {
      unsigned long value = mem_[(address + i) & mask_];
      snprintf(buf, 64, "0x%08lX: 0x%02lX\n\r", address + i, value);
      DEBUG_SERIAL.print(buf);
    }
  };

  void randomize_memory(uint32_t seed) override {
    base_seed_ = seed;
    if (!mem_) {
      return;
    }
    // Fill memory with random data, 32-bits at a time for speed.
    uint32_t *fast_ptr = reinterpret_cast<uint32_t*>(mem_);

    for (size_t i = 0; i < (size_ / 4); i++) {
 
      uint32_t random_u32 = static_cast<uint32_t>(random(__LONG_MAX__));
      // This was a debug to verify that randomization was actually working.
      // if (i < 10) {
      //   DEBUG_SERIAL.print("## SDRAM: Randomizing memory value ");
      //   DEBUG_SERIAL.println(random_u32, HEX);
      // }
      fast_ptr[i] = random_u32;
    }
  };

  void set_strategy(DefaultStrategy strategy, uint32_t start, uint32_t end) override {
    if (start < strategy_start_ || end > size_) {
      DEBUG_SERIAL.println("## SDRAM: Invalid strategy range");
      return;
    }
    strategy_start_ = start;
    strategy_end_ = end;
    base_seed_ = 0x1024; // Reset base seed for new strategy.
  };

private:
  size_t   size_;
  size_t   mask_;
  uint8_t* mem_;
  uint32_t base_seed_;
  uint32_t strategy_start_ = 0x1024; // Address below which to ignore strategy.
  uint32_t strategy_end_ = 0xFFFFFF; // Address above which to ignore strategy.

  uint16_t gen_default_u16(DefaultStrategy strategy) {
    // Generate a default 16-bit value based on the strategy.
    switch (strategy) {
      case DefaultStrategy::Zero:
        return 0x0000;
      case DefaultStrategy::Ones:
        return 0xFFFF;
      case DefaultStrategy::Random:
        return gen_random_u16(0);
      default:
        return 0x0000; // Fallback to zero if unknown strategy
    }
  }

  uint16_t gen_random_u16(uint32_t address) {
    // Generate a pseudo-random 16-bit value based on the address and base seed.
    uint32_t seed = base_seed_ ^ address;
    randomSeed(seed);
    return static_cast<uint16_t>(random(std::numeric_limits<uint16_t>::max()));
  }
};
