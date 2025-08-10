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

#include "HatBase.h"
#include "../serial_config.h"
#include "../gpio_pins.h"

#define CPU_186

#define USE_SMI 0
#define WRITE_CYCLE T3
#define ADDRESS_SPACE_MASK 0xFFFFF

#define STORE_IO_BASE 0x0000

#define WRITE_BIT(data, mask, set_macro, clear_macro) \
    do { if ((data) & (mask)) { set_macro; } else { clear_macro; } } while (0)

#define READY_PIN 6
#define TEST_PIN 7
#define LOCK_PIN 10
#define INTR_PIN 12
#define NMI_PIN 13

// How many cycles to hold the RESET signal high. Intel says "greater than 4" although 4 seems to work.
#define RESET_HOLD_CYCLE_COUNT 30
// How many cycles it takes to reset the CPU after RESET signal goes low. First ALE should occur after this many cycles.
#define RESET_CYCLE_COUNT 35
// If we didn't see an ALE after this many cycles, give up
#define RESET_CYCLE_TIMEOUT 45
// What logic level RESET is when asserted
#define RESET_ASSERT 0
// What logic level RESET is when deasserted
#define RESET_DEASSERT 1
#define EMULATE_8288 0
#define HAVE_QUEUE_STATUS 0

#define ADDRESS_SPACE_MASK 0xFFFFF // 20-bit address space for 80186
#define WRITE_CYCLE T3
#define STORE_TIMEOUT 1000
#define LOAD_TIMEOUT 1000

#define READ_SMI_PIN 1

class Hat80186 : public HatBase<Hat80186> {
private:
  // Address pins, used for slow address reading via digitalRead()
  static constexpr std::array<int,20> ADDRESS_PINS = {{
    22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41
  }};
  const int ADDRESS_LINES = 20;

  // All output pins, used to set pin direction on setup
  static constexpr std::array<int,8> OUTPUT_PINS = {{
    4,  // CLK
    5,  // RESET
    6,  // READY
    7,  // TEST
    12, // INTR
    13, // NMI,
    54, // AEN,
    55, // CEN
  }};

  // All input pins, used to set pin direction on setup
  static constexpr std::array<int,40> INPUT_PINS = {{
    3,8,9,10,11,14,15,16,17,
    22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,
    43,44,45,46,47,48,49,50,51,52,53
  }};

  BusDirection busDirection = BusDirection::Input; // Default bus direction is input
  size_t dataBusWidth = 16; // Default data bus width is 16 bits
  size_t addressBusWidth = 22; // Default address bus width is 22 bits

protected:
  static constexpr unsigned ClockDivisor = 2;

public:   
  static void initPins() {
    // Set all output pins to OUTPUT}
    for (size_t p = 0; p < (sizeof OUTPUT_PINS / sizeof OUTPUT_PINS[0]); p++) {
      pinMode(OUTPUT_PINS[p], OUTPUT);
    }
    // Set all input pins to INPUT
    for (size_t p = 0; p < (sizeof INPUT_PINS / sizeof INPUT_PINS[0]); p++) {
      pinMode(INPUT_PINS[p], INPUT);
    }

    // RQ pin is temporarily borrowed for RESET_OUT sampling
    pinMode(3, INPUT);
    digitalWrite(3, LOW);  // Disable pull-up?

    digitalWrite(TEST_PIN, LOW);  // No FPU supported on 80186
    digitalWrite(INTR_PIN, LOW);  // Must set these to a known value or risk spurious interrupts!
    digitalWrite(NMI_PIN, LOW);   // Must set these to a known value or risk spurious interrupts!
    //digitalWrite(AEN_PIN, LOW);   // AEN is enable-low
    //digitalWrite(CEN_PIN, HIGH);  // Command enable enables the outputs on the i8288
  }

  inline void cpuTick() {
    WRITE_PIN_D04(1);
    if (ClockHighDelay > 0) {
      delayMicroseconds(ClockHighDelay);
    }
    WRITE_PIN_D04(0);
    if (ClockLowDelay > 0) {
      delayMicroseconds(ClockLowDelay);
    }
    WRITE_PIN_D04(1);
    if (ClockHighDelay > 0) {
      delayMicroseconds(ClockHighDelay);
    }
    WRITE_PIN_D04(0);
    if (ClockLowDelay > 0) {
      delayMicroseconds(ClockLowDelay);
    }
  }

  uint16_t readDataBus(ActiveBusWidth width) {
    setBusDirection(BusDirection::Input, width);
    uint16_t data = 0;
    if ((width == ActiveBusWidth::EightLow) || (width == ActiveBusWidth::Sixteen)) {
      // Read data from bus pins
      if (READ_DBUS_00) data |= 0x0001;
      if (READ_DBUS_01) data |= 0x0002;
      if (READ_DBUS_02) data |= 0x0004;
      if (READ_DBUS_03) data |= 0x0008;
      if (READ_DBUS_04) data |= 0x0010;
      if (READ_DBUS_05) data |= 0x0020;
      if (READ_DBUS_06) data |= 0x0040;
      if (READ_DBUS_07) data |= 0x0080;
    }
    if ((width == ActiveBusWidth::EightHigh) || (width == ActiveBusWidth::Sixteen)) {
      if (READ_DBUS_08) data |= 0x0100;
      if (READ_DBUS_09) data |= 0x0200;
      if (READ_DBUS_10) data |= 0x0400;
      if (READ_DBUS_11) data |= 0x0800;
      if (READ_DBUS_12) data |= 0x1000;
      if (READ_DBUS_13) data |= 0x2000;
      if (READ_DBUS_14) data |= 0x4000;
      if (READ_DBUS_15) data |= 0x8000;
    }
    return data;
  }

  /// @brief Return true if the current hat has a multiplexed bus.
  static bool hasMultiplexedBusImpl() {
    return true;
  }

  void writeDataBus(uint16_t data, ActiveBusWidth width) {
    
    #if defined(__SAM3X8E__) // If Arduino DUE

      setBusDirection(BusDirection::Output, width);

      if ((width == EightLow) || (width == Sixteen)) {
        // Write low-order byte to data bus pins
        (data & 0x01) ? PIOB->PIO_SODR = BIT26 : PIOB->PIO_CODR = BIT26;      // Pin 22
        (data & 0x02) ? PIOA->PIO_SODR = BIT14 : PIOA->PIO_CODR = BIT14;      // Pin 23
        (data & 0x04) ? PIOA->PIO_SODR = BIT15 : PIOA->PIO_CODR = BIT15;      // Pin 24
        (data & 0x08) ? PIOD->PIO_SODR = BIT00 : PIOD->PIO_CODR = BIT00;      // Pin 25
        (data & 0x10) ? PIOD->PIO_SODR = BIT01 : PIOD->PIO_CODR = BIT01;      // Pin 26
        (data & 0x20) ? PIOD->PIO_SODR = BIT02 : PIOD->PIO_CODR = BIT02;      // Pin 27
        (data & 0x40) ? PIOD->PIO_SODR = BIT03 : PIOD->PIO_CODR = BIT03;      // Pin 28
        (data & 0x80) ? PIOD->PIO_SODR = BIT06 : PIOD->PIO_CODR = BIT06;      // Pin 29
      }

      if ((width == EightHigh) || (width == Sixteen)) {
        (data & 0x0100) ? PIOD->PIO_SODR = BIT09 : PIOD->PIO_CODR = BIT09;    // AD8 Pin 30 (PD9)
        (data & 0x0200) ? PIOA->PIO_SODR = BIT07 : PIOA->PIO_CODR = BIT07;    // AD9 Pin 31 (PA7)
        (data & 0x0400) ? PIOD->PIO_SODR = BIT10 : PIOD->PIO_CODR = BIT10;    // AD10 Pin 32 (PD10)
        (data & 0x0800) ? PIOC->PIO_SODR = BIT01 : PIOC->PIO_CODR = BIT01;    // AD11 Pin 33 (PC1)
        (data & 0x1000) ? PIOC->PIO_SODR = BIT02 : PIOC->PIO_CODR = BIT02;    // AD12 Pin 34 (PC2)
        (data & 0x2000) ? PIOC->PIO_SODR = BIT03 : PIOC->PIO_CODR = BIT03;    // AD13 Pin 35 (PC3)
        (data & 0x4000) ? PIOC->PIO_SODR = BIT04 : PIOC->PIO_CODR = BIT04;    // AD14 Pin 36 (PC4)
        (data & 0x8000) ? PIOC->PIO_SODR = BIT05 : PIOC->PIO_CODR = BIT05;    // AD15 Pin 37 (PC5)
      }
    #elif defined(ARDUINO_GIGA)
      setBusDirection(BusDirection::Output, width);

      if ((width == ActiveBusWidth::EightLow) || (width == ActiveBusWidth::Sixteen)) {
        WRITE_BIT(data, 0x01, SET_DBUS_00, CLEAR_DBUS_00);
        WRITE_BIT(data, 0x02, SET_DBUS_01, CLEAR_DBUS_01);
        WRITE_BIT(data, 0x04, SET_DBUS_02, CLEAR_DBUS_02);
        WRITE_BIT(data, 0x08, SET_DBUS_03, CLEAR_DBUS_03);
        WRITE_BIT(data, 0x10, SET_DBUS_04, CLEAR_DBUS_04);
        WRITE_BIT(data, 0x20, SET_DBUS_05, CLEAR_DBUS_05);
        WRITE_BIT(data, 0x40, SET_DBUS_06, CLEAR_DBUS_06);
        WRITE_BIT(data, 0x80, SET_DBUS_07, CLEAR_DBUS_07);
      }
      
      if ((width == ActiveBusWidth::EightHigh) || (width == ActiveBusWidth::Sixteen)) {
        WRITE_BIT(data, 0x0100, SET_DBUS_08, CLEAR_DBUS_08);
        WRITE_BIT(data, 0x0200, SET_DBUS_09, CLEAR_DBUS_09);
        WRITE_BIT(data, 0x0400, SET_DBUS_10, CLEAR_DBUS_10);
        WRITE_BIT(data, 0x0800, SET_DBUS_11, CLEAR_DBUS_11);
        WRITE_BIT(data, 0x1000, SET_DBUS_12, CLEAR_DBUS_12);
        WRITE_BIT(data, 0x2000, SET_DBUS_13, CLEAR_DBUS_13);
        WRITE_BIT(data, 0x4000, SET_DBUS_14, CLEAR_DBUS_14);
        WRITE_BIT(data, 0x8000, SET_DBUS_15, CLEAR_DBUS_15);        
      }
    #endif
  }

    /// Read the multiplexed address bus. Returns a 20-bit value.
  uint32_t readAddressBus(bool peek = true) {
    // If we're not peeking, set the bus direction to input
    if (!peek) {
      setBusDirection(BusDirection::Input, ActiveBusWidth::Sixteen);
    }

    uint32_t address = 0;
    // Read the address bus pins
    if (READ_ABUS_00) address |= 0x00000001;  // AD0  Pin 22
    if (READ_ABUS_01) address |= 0x00000002;  // AD1  Pin 23
    if (READ_ABUS_02) address |= 0x00000004;  // AD2  Pin 24
    if (READ_ABUS_03) address |= 0x00000008;  // AD3  Pin 25
    if (READ_ABUS_04) address |= 0x00000010;  // AD4  Pin 26
    if (READ_ABUS_05) address |= 0x00000020;  // AD5  Pin 27
    if (READ_ABUS_06) address |= 0x00000040;  // AD6  Pin 28
    if (READ_ABUS_07) address |= 0x00000080;  // AD7  Pin 29
    if (READ_ABUS_08) address |= 0x00000100;  // AD8  Pin 30
    if (READ_ABUS_09) address |= 0x00000200;  // AD9  Pin 31
    if (READ_ABUS_10) address |= 0x00000400;  // AD10 Pin 32
    if (READ_ABUS_11) address |= 0x00000800;  // AD11 Pin 33
    if (READ_ABUS_12) address |= 0x00001000;  // AD12 Pin 34
    if (READ_ABUS_13) address |= 0x00002000;  // AD13 Pin 35
    if (READ_ABUS_14) address |= 0x00004000;  // AD14 Pin 36
    if (READ_ABUS_15) address |= 0x00008000;  // AD15 Pin 37
    if (READ_ABUS_16) address |= 0x00010000;  // AD16 Pin 38
    if (READ_ABUS_17) address |= 0x00020000;  // AD17 Pin 39
    if (READ_ABUS_18) address |= 0x00040000;  // AD18 Pin 40
    if (READ_ABUS_19) address |= 0x00080000;  // AD19 Pin 41
    return address;
  }
};