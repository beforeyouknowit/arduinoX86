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
// Define the baud rate to use for the RS232 module plugged into your hat
// (or possibly directly into your Arduino, but in this case our "hat" is
// the specific pinout template.

#include <config.h>

class Hat8088;
class Hat80186;
class Hat80286;
class Hat80386;

template<typename Hat>
struct HatTraits {
  static constexpr unsigned long kDebugBaudRate = DEBUG_BAUD_RATE; // Default baud rate
};

// Specialize for Hat8088
template<>
struct HatTraits<Hat8088> {
  static constexpr unsigned long kDebugBaudRate = DEBUG_BAUD_RATE;
};

// Specialize for Hat80186
template<>
struct HatTraits<Hat80186> {
  static constexpr unsigned long kDebugBaudRate = DEBUG_BAUD_RATE;
};

// Specialize for Hat80286
template<>
struct HatTraits<Hat80286> {
  static constexpr unsigned long kDebugBaudRate = DEBUG_BAUD_RATE;
};

// Specialize for Hat80386
template<>
struct HatTraits<Hat80386> {
  static constexpr unsigned long kDebugBaudRate = DEBUG_BAUD_RATE;
};