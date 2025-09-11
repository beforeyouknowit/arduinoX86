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
#include "DebugFilter.h"

constexpr const char* getColor(DebugType stage) {
  using namespace ansi;

  switch (stage) {
    case DebugType::STATE:     return yellow;
    case DebugType::RESET:     return red;
    case DebugType::SETUP:     return magenta;
    case DebugType::VECTOR:    return cyan;
    case DebugType::ID:        return green;
    case DebugType::LOAD:      return bright_green;
    case DebugType::LOAD_DONE: return bright_blue;
    case DebugType::EXECUTE:   return bright_yellow;
    case DebugType::STORE:     return bright_red;
    case DebugType::FINALIZE:  return bright_white;  // or bright_black if you have it
    case DebugType::INSTR:     return bright_cyan;
    case DebugType::EMU:       return bright_magenta;
    case DebugType::QUEUE:     return white;
    case DebugType::TSTATE:    return blue;
    case DebugType::PIN_CMD:   return green;
    case DebugType::BUS:       return cyan;
    case DebugType::PROTO:     return yellow;
    case DebugType::CMD:       return red;
    default:                  return reset;
  }
}

// Debug print mixin templated on Serial port type
template<typename SerialPort>
class DebugPrintMixin {
protected:
  SerialPort& serial;
  DebugFilter filter;

public:
  explicit DebugPrintMixin(SerialPort& s) : serial(s) {}

  template<typename T>
  // String overload
  inline void debugPrint(DebugType stage, const char* text) {
    if (!filter.isEnabled(stage)) return;
    serial.print(getColor(stage));
    serial.print(text);
    serial.print(ansi::reset);
  }

  // Generic value overload
  template<typename T>
  inline void debugPrint(DebugType stage, T value) {
    if (!filter.isEnabled(stage)) return;
    serial.print(getColor(stage));
    serial.print(value);
    serial.print(ansi::reset);
  }

  // Generic value with base (for integers)
  template<typename T>
  inline void debugPrint(DebugType stage, T value, int base) {
    if (!filter.isEnabled(stage)) return;
    serial.print(getColor(stage));
    serial.print(value, base);
    serial.print(ansi::reset);
  }

  // --- Print with newline ---

  // String overload
  inline void debugPrintln(DebugType stage, const char* text) {
    if (!filter.isEnabled(stage)) return;
    serial.print(getColor(stage));
    serial.println(text);
    serial.print(ansi::reset);
  }

  // Generic value overload
  template<typename T>
  inline void debugPrintln(DebugType stage, T value) {
    if (!filter.isEnabled(stage)) return;
    serial.print(getColor(stage));
    serial.println(value);
    serial.print(ansi::reset);
  }

  // Generic value with base
  template<typename T>
  inline void debugPrintln(DebugType stage, T value, int base) {
    if (!filter.isEnabled(stage)) return;
    serial.print(getColor(stage));
    serial.println(value, base);
    serial.print(ansi::reset);
  }

  // Println with no arguments, just newline with color
  inline void debugPrintln(DebugType stage) {
    if (!filter.isEnabled(stage)) return;
    serial.print(getColor(stage));
    serial.println();
    serial.print(ansi::reset);
  }
};