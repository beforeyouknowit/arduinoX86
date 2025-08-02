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

#include <BusTypes.h>

enum class FpuType : uint8_t {
  noFpu,
  i8087,
};

enum class CpuBusWidth : uint8_t {
  Eight,
  Sixteen,
};

// Type of CPU. These are either detected or specified by the configured hat.
enum class CpuType : uint8_t {
  Undetected,
  i8088, 
  i8086,
  necV20,
  necV30,
  i80188,
  i80186,
  i80286,
  i80386,
};

struct CpuResetResult {
  bool success;         // True if reset was successful
  BusWidth busWidth;    // The bus width detected (Eight or Sixteen)
  bool queueStatus;     // True if queue status lines are available
};

