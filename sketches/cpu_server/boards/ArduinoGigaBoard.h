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
#include "../gpio_pins.h"    // Your GPIO macros like WRITE_PIN_D04, etc.
#include "../hat_config.h"   // Board-specific configuration and pin mappings

#include "../DebugPrint.h"   // Debug print mixin
#include "../hats/HatTraits.h" // Hat-specific traits

template<typename Hat>
class ArduinoGigaBoard : public DebugPrintMixin<decltype(Serial1)> {
public:
  ArduinoGigaBoard() : DebugPrintMixin(Serial1) {}

  void init() {
    pinMode(4, OUTPUT);  // CLK_PIN (could also be from BoardTraits if you want)
    // Initialize other pins as needed...

    Serial1.begin(HatTraits<Hat>::kDebugBaudRate);
  }

  inline void clockHighDelay() {
    // Board-specific tuning. in this case do nothing!
  }

  inline void clockLowDelay() {
    // Board-specific timing. In this case do nothing!
  }

  void digitalWritePin(int pin, bool val) {
    digitalWrite(pin, val ? HIGH : LOW);
  }

  bool digitalReadPin(int pin) {
    return digitalRead(pin) == HIGH;
  }


};