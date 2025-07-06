/*
    ArduinoX86 Copyright 2022-2025 Daniel Balsom
    https://github.com/dbalsom/arduinoX86

    Permission is hereby granted, free of charge, to any person obtaining a
    copy of this software and associated documentation files (the "Software"),
    to deal in the Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, sublicense,
    and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER   
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.
*/

#include <cstdint>
#include <config.h>
#include <serial_config.h>

extern uint8_t PACKET_BUFFER[PACKET_SIZE]; // Packet buffer for serial communication

#if defined(HAT_8088_V1)
#include <hats/Hat8088.h>

// Define the static constexpr members of Hat8088
constexpr std::array<int,8> Hat8088::OUTPUT_PINS;
constexpr std::array<int,40> Hat8088::INPUT_PINS;
#endif

#if defined(HAT_286_5V_V1)
#include <hats/Hat80286.h>

// Define the static constexpr members of Hat80286
constexpr std::array<int,6> Hat80286::OUTPUT_PINS;
constexpr std::array<int,36> Hat80286::INPUT_PINS;
#endif

#if defined(HAT_386_3V_V1)
#include <hats/Hat80386.h>

// Define the static constexpr members of Hat80286
constexpr std::array<int,7> Hat80386::OUTPUT_PINS;
constexpr std::array<int,28> Hat80386::INPUT_PINS;
#endif