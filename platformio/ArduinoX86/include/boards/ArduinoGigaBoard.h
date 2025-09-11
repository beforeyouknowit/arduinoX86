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

#include <config.h>
#include <gpio_pins.h>
#include <serial_config.h>
#include <DebugPrint.h>

#include "../shields/ShieldTraits.h"

template<typename Shield>
class ArduinoGigaBoard : public DebugPrintMixin<decltype(Serial2)> {
public:
  ArduinoGigaBoard() : DebugPrintMixin(Serial2) {}

  void init() {

    // Initialize SDRAM
    SDRAM.begin();

    // Initialize GPIO via the shield
    Shield::initPins();

    // Initialize the Serial2 port for debugging.
    Serial2.begin(ShieldTraits<Shield>::kDebugBaudRate);
    while (!Serial2)
      ;

    // Initialize the board's debugging states. 
    // Warning, Error, and Emit are always enabled.
    setDebugType(DebugType::STATE,     DEBUG_STATE);
    setDebugType(DebugType::RESET,     DEBUG_RESET);
    setDebugType(DebugType::SETUP,     DEBUG_SETUP);
    setDebugType(DebugType::VECTOR,    DEBUG_VECTOR);
    setDebugType(DebugType::ID,        DEBUG_ID);
    setDebugType(DebugType::LOAD,      DEBUG_LOAD);
    setDebugType(DebugType::LOAD_DONE, DEBUG_LOAD_DONE);
    setDebugType(DebugType::EXECUTE,   DEBUG_EXECUTE);
    setDebugType(DebugType::STORE,     DEBUG_STORE);
    setDebugType(DebugType::FINALIZE,  DEBUG_FINALIZE);
    setDebugType(DebugType::INSTR,     DEBUG_INSTR);
    setDebugType(DebugType::EMU,       DEBUG_EMU);
    setDebugType(DebugType::QUEUE,     DEBUG_QUEUE);
    setDebugType(DebugType::TSTATE,    DEBUG_TSTATE);
    setDebugType(DebugType::PIN_CMD,   DEBUG_PIN_CMD);
    setDebugType(DebugType::BUS,       DEBUG_BUS);
    setDebugType(DebugType::PROTO,     DEBUG_PROTO);
    setDebugType(DebugType::CMD,       DEBUG_CMD);
    setDebugType(DebugType::DUMP,      DEBUG_DUMP);
    setDebugType(DebugType::SERVER,    DEBUG_SERVER);
    setDebugType(DebugType::WARNING,   true);
    setDebugType(DebugType::ERROR,     true);
    setDebugType(DebugType::EMIT,      true);
  }
};