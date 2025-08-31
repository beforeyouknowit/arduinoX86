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

#include "ansi_color.h"
#include <DebugFilter.h>

constexpr const char* getColor(DebugType stage) {
  using namespace ansi;

  switch (stage) {
    case DebugType::WARNING:   return bright_yellow;
    case DebugType::ERROR:     return red;
    case DebugType::STATE:     return yellow;
    case DebugType::RESET:     return green;
    case DebugType::SETUP:     return cyan;
    case DebugType::VECTOR:    return bright_cyan;
    case DebugType::ID:        return green;
    case DebugType::LOAD:      return bright_blue;
    case DebugType::LOAD_DONE: return bright_blue;
    case DebugType::EXECUTE:   return bright_yellow;
    case DebugType::STORE:     return bright_magenta;
    case DebugType::FINALIZE:  return blue;
    case DebugType::INSTR:     return bright_cyan;
    case DebugType::EMU:       return bright_magenta;
    case DebugType::QUEUE:     return bright_white;
    case DebugType::TSTATE:    return blue;
    case DebugType::PIN_CMD:   return green;
    case DebugType::BUS:       return cyan;
    case DebugType::PROTO:     return yellow;
    case DebugType::CMD:       return bright_cyan;
    case DebugType::DUMP:      return bright_yellow;
    case DebugType::SERVER:    return bright_green;
    default:                   return reset;
  }
}

// Debug print mixin templated on Serial port type
template<typename SerialPort>
class DebugPrintMixin {

private:
  static constexpr size_t BUFFER_SIZE = 256;
  char buffer_[BUFFER_SIZE];
  char printf_buffer_[BUFFER_SIZE];
  char *buffer_ptr_ = buffer_;
  size_t buffer_remain_ = BUFFER_SIZE;
  char have_deferred_buffer_ = false;
  bool enabled_ = true;

protected:
  SerialPort& serial;
  DebugFilter filter;

public:
  explicit DebugPrintMixin(SerialPort& s) : serial(s) {
    buffer_[0] = '\0';
    printf_buffer_[0] = '\0';
  }

  void setDebugType(DebugType stage, bool enabled) {
    filter.setTypeEnabled(stage, enabled);
  }

  void debugPrintf(DebugType stage, bool defer, const char* fmt, ...);

  // String overload
  inline void debugPrint(DebugType stage, const char* text, bool defer = false) {
    if (!serial || !filter.isEnabled(stage) || !enabled_) return;

    if (defer) {
      // Defer printing to avoid blocking
      snprintf(buffer_ptr_, buffer_remain_, "%s%s%s", getColor(stage), text, ansi::reset);
      buffer_ptr_ += strlen(buffer_ptr_);
      buffer_remain_ -= strlen(buffer_ptr_);
      have_deferred_buffer_ = true;
      return;
    }
    serial.print(getColor(stage));
    serial.print(text);
    serial.print(ansi::reset);
  }

  // Generic value overload
  template<typename T>
  inline void debugPrint(DebugType stage, T value, bool defer = false) {
    if (!serial || !filter.isEnabled(stage) || !enabled_) return;

    if(defer) {
      String s = String(value);
      snprintf(buffer_ptr_, buffer_remain_, "%s%s%s", getColor(stage), s.c_str(), ansi::reset);
      buffer_ptr_ += strlen(buffer_ptr_);
      buffer_remain_ -= strlen(buffer_ptr_);
      have_deferred_buffer_ = true;
      return;
    }
    serial.print(getColor(stage));
    serial.print(value);
    serial.print(ansi::reset);
  }

  // Generic value with base (for integers)
  
  template<typename T>
  inline void debugPrint(DebugType stage, T value, int base, bool defer = false) {
    if (!serial || !filter.isEnabled(stage) || !enabled_) return;
    if(defer) {
      // Must convert base
      if (base == 16) {
        snprintf(buffer_ptr_, buffer_remain_, "%s%lx%s", getColor(stage), static_cast<unsigned long>(value), ansi::reset);
      } else {
        snprintf(buffer_ptr_, buffer_remain_, "%s%ld%s", getColor(stage), static_cast<long>(value), ansi::reset);
      }
      buffer_ptr_ += strlen(buffer_ptr_);
      buffer_remain_ -= strlen(buffer_ptr_);
      have_deferred_buffer_ = true;
      return;
    }
    serial.print(getColor(stage));
    serial.print(value, base);
    serial.print(ansi::reset);
  }

  // --- Print with newline ---

  // String overload
  inline void debugPrintln(DebugType stage, const char* text, bool defer = false) {
    if (!serial || !filter.isEnabled(stage) || !enabled_) return;

    if (defer) {
      snprintf(buffer_ptr_, buffer_remain_, "%s%s%s\n\r", getColor(stage), text, ansi::reset);
      buffer_ptr_ += strlen(buffer_ptr_);
      buffer_remain_ -= strlen(buffer_ptr_);
      have_deferred_buffer_ = true;
      return;
    }
    serial.print(getColor(stage));
    serial.println(text);
    serial.print(ansi::reset);
  }

  // Generic value overload
  template<typename T>
  inline void debugPrintln(DebugType stage, T value, bool defer = false) {
    if (!serial || !filter.isEnabled(stage) || !enabled_) return;

    if (defer) {
      // Convert value to string
      String s = String(value);
      snprintf(buffer_ptr_, buffer_remain_, "%s%s%s\n\r", getColor(stage), s.c_str(), ansi::reset);
      buffer_ptr_ += strlen(buffer_ptr_);
      buffer_remain_ -= strlen(buffer_ptr_);
      have_deferred_buffer_ = true;
      return;
    }
    serial.print(getColor(stage));
    serial.println(value);
    serial.print(ansi::reset);
  }

  // Generic value with base
  template<typename T>
  inline void debugPrintln(DebugType stage, T value, int base, bool defer = false) {
    if (!serial || !filter.isEnabled(stage) || !enabled_) return;

    if (defer) {
      // Must convert base
      if (base == 16) {
        snprintf(buffer_ptr_, buffer_remain_, "%s%lx%s\n\r", getColor(stage), static_cast<unsigned long>(value), ansi::reset);
      } else {
        snprintf(buffer_ptr_, buffer_remain_, "%s%ld%s\n\r", getColor(stage), static_cast<long>(value), ansi::reset);
      }
      buffer_ptr_ += strlen(buffer_ptr_);
      buffer_remain_ -= strlen(buffer_ptr_);
      have_deferred_buffer_ = true;
      return;
    }
    serial.print(getColor(stage));
    serial.println(value, base);
    serial.print(ansi::reset);
  }

  // Println with no arguments, just newline with color
  inline void debugPrintln(DebugType stage, bool defer = false) {
    if (!serial || !filter.isEnabled(stage) || !enabled_) return;

    if (defer) {
      // Defer printing to avoid blocking
      snprintf(buffer_ptr_, buffer_remain_, "\n\r");
      buffer_ptr_ += strlen(buffer_ptr_);
      buffer_remain_ -= strlen(buffer_ptr_);
      have_deferred_buffer_ = true;
      return;
    }
    serial.println();
  }

  inline void debugPrintDeferred() {
    if (!have_deferred_buffer_) return;

    serial.print(buffer_);
    // Reset buffer
    have_deferred_buffer_ = false;
    buffer_ptr_ = buffer_;
    buffer_remain_ = BUFFER_SIZE;
    buffer_[0] = '\0'; 
  }

  inline bool haveDeferredBuffer() {
    return have_deferred_buffer_;
  }

  inline void setDebugEnabled(bool enabled) {
    enabled_ = enabled;
  }

  inline bool isDebugEnabled() const {
    return enabled_;
  }
};