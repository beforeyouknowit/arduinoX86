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

/// @brief Type of bus transfer. This is can be Code, Memory, or Io.
enum BusTransferType {
  Code,
  Memory,
  Io,
};

/// @brief Direction of the data bus from the Arduino's perspective.
/// - Input: The Arduino reads data from the bus (CPU is writing).
/// - Output: The Arduino writes data to the bus (CPU is reading).
enum class BusDirection {
  Input,
  Output
};

/// @brief Data bus width used by the CPU.
enum class BusWidth : uint8_t {
  Eight = 0, ///< 8-bit bus width
  Sixteen = 1, ///< 16-bit bus width
};

/// @brief Currently active data bus width. There are three possible data bus states:
/// - EightLow: the low 8 bits are active,
/// - EightHigh: the high 8 bits are active,
/// - Sixteen: all 16 bits are active
enum class ActiveBusWidth : uint8_t {
  EightLow,
  EightHigh,
  Sixteen,
};

// Bus transfer states, as determined by status lines S0-S2.
enum BusStatus {
  IRQA = 0,   // IRQ Acknowledge
  IOR  = 1,   // IO Read
  IOW  = 2,   // IO Write
  HALT = 3,   // Halt
  CODE = 4,   // Code
  MEMR = 5,   // Memory Read
  MEMW = 6,   // Memory Write
  PASV = 7    // Passive
};

// Bus transfer cycles. Tw is wait state, inserted if READY is not asserted during T3.
enum TCycle { 
  TI = 0,
  T1 = 1,
  T2 = 2,
  T3 = 3,
  T4 = 4,
  TW = 5,
};