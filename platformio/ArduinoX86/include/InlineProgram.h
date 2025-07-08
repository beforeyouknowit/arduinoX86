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

/// @file InlineProgram.h
/// @brief An InlineProgram represents a relocatable sequence of opcode bytes 
/// that can be fed to a CPU for execution regardless of IP.

#pragma once

#include <cstddef>
#include <cstdint>
#include <initializer_list>
#include <vector>

#include <BusTypes.h>
#include <DebugFilter.h>
#include <serial_config.h>

class InlineProgram {
public:

  constexpr static uint16_t DOUBLE_NOP = 0x9090;

  InlineProgram(const char *name, std::initializer_list<uint8_t> init, size_t vector_patch_offset = 0, uint16_t fill_data = DOUBLE_NOP)
    : name_(name),program_(init), len_(program_.size()), pc_(0), fillData_(fill_data), vectorPatchOffset_(vector_patch_offset)
  {}

  void reset() { pc_ = 0; }

  void set_pc(size_t pc) { 
    if (pc < len_) {
      pc_ = pc; 
    } else {
      pc_ = len_; // Set to end if out of bounds
    }
  }

  uint16_t read(uint32_t address, ActiveBusWidth width) {
    bool a0 = (address & 1) == 1;
    uint16_t data = fillData_;
    if (pc_ >= len_) {
      return data;
    }

    if (width == ActiveBusWidth::EightLow) {
      // Reading a byte at an even address. We can just return the byte.
      // The upper half of the data bus is not valid.
      data = program_[pc_++];
    } 
    else if (width == ActiveBusWidth::EightHigh) {
      // TODO: bounds checks
      //DEBUG_SERIAL.println("## Odd read ##");
      if (pc_ > 0) {
        // This byte doesn't really matter, but we can simulate fetching more realistically by including it.
        // If this happens to be the start of the program though, it will just have to be 0.
        data = program_[pc_ - 1];
      }
      if (pc_ < len_) {
        data &= 0x00FF;
        data |= ((uint16_t)program_[pc_++]) << 8;
      }
    } 
    else {
      // 16-bit read.
      if (!a0) {
        // 16-bit read at even address
        //DEBUG_SERIAL.println("## Even read ##");
        
        // Read the first byte (low byte)
        data = program_[pc_++];
        // Read the second byte (high byte) if we have one. 
        // If we are at the end of the program, the default high byte of NO_PROGRAM will be used.
        // (Currently 0x90)
        if (pc_ < len_) {
          data &= 0x00FF;
          data |= ((uint16_t)program_[pc_++]) << 8;
        }
      } else {
        // 16-bit read at odd address. This shouldn't be possible...
        DEBUG_SERIAL.println("## read_program(): Odd 16-bit read, shouldn't happen! ##");
      }
    }
    return data;
  }

  uint16_t read_at(uint32_t base, uint32_t address, ActiveBusWidth width) {
    bool a0 = (address & 1) == 1;
    uint16_t data = fillData_;

    // Calculate the offset.
    size_t offset = address - base;

    // Check if the offset is within the program length.
    if (offset >= len_) {
      return data;
    }

    if (width == ActiveBusWidth::EightLow) {
      // Reading a byte at an even address. We can just return the byte.
      // The upper half of the data bus is not valid.
      data = program_[offset];
    } 
    else if (width == ActiveBusWidth::EightHigh) {
      // TODO: bounds checks
      //DEBUG_SERIAL.println("## Odd read ##");
      if (offset > 0) {
        // This byte doesn't really matter, but we can simulate fetching more realistically by including it.
        // If this happens to be the start of the program though, it will just have to be 0.
        data = program_[offset - 1];
      }
      if (pc_ < len_) {
        data &= 0x00FF;
        data |= ((uint16_t)program_[offset]) << 8;
      }
    } 
    else {
      // 16-bit read.
      if (!a0) {
        // 16-bit read at even address
        //DEBUG_SERIAL.println("## Even read ##");
        
        // Read the first byte (low byte)
        data = program_[offset];
        // Read the second byte (high byte) if we have one. 
        // If we are at the end of the program, the default high byte of the program fill will be used.
        // (Usually 0x90, may differ in 8080 emulation mode)
        if ((offset + 1) < len_) {
          data &= 0x00FF;
          data |= ((uint16_t)program_[offset + 1]) << 8;
        }
      } else {
        // 16-bit read at odd address. This shouldn't be possible...
        DEBUG_SERIAL.println("## read_program(): Odd 16-bit read, shouldn't happen! ##");
      }
    }

    return data;
  }

  void write_u16_at(size_t offset, uint16_t data) {
    if (offset + 1 >= len_) {
      return; // Out of bounds
    }
    program_[offset] = data & 0xFF;
    program_[offset + 1] = (data >> 8) & 0xFF;
  }

  size_t len() const { return len_; }
  size_t remaining() const { return (pc_ < len_) ? (len_ - pc_) : 0; }
  bool has_remaining() const { return pc_ < len_; }
  const char *name() const { return name_; }
  size_t pc() const { return pc_; }
  void setPc(size_t p) { pc_ = (p < len_) ? p : len_; }
  size_t vector_offset() const { return vectorPatchOffset_; }
  void patch_vector(uint16_t segment) {
    if (vectorPatchOffset_ + 1 < len_) {
      program_[vectorPatchOffset_] = segment & 0xFF;
      program_[vectorPatchOffset_ + 1] = (segment >> 8) & 0xFF;
    }
  }

  template<typename Board>
  void debug_print(Board& board, DebugType dtype, const char* prefix, uint16_t value) const {
    board.debugPrint(dtype, prefix, true);
    board.debugPrint(dtype, ": writing ", true);
    board.debugPrint(dtype, name(), true);
    board.debugPrint(dtype, " program to bus: ", true);
    board.debugPrint(dtype, value, 16, true);
    board.debugPrint(dtype, " new pc: ", true);
    board.debugPrint(dtype, pc(), true);
    board.debugPrint(dtype, "/", true);
    board.debugPrintln(dtype, len(), true);
  }

private:
  const char *name_;
  std::vector<uint8_t> program_;
  size_t len_;
  size_t pc_;

  uint16_t fillData_;
  size_t vectorPatchOffset_;
};  