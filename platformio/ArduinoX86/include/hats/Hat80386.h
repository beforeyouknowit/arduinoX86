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

// Forward declaration of cycle() function
void cycle();

#include <arduinoX86.h>
#include <serial_config.h>
#include <gpio_pins.h>
#include <BusTypes.h>
#include <hats/HatBase.h>
#include <hats/Pins.h>
#include <DebugFilter.h>

#define WRITE_BIT(data, mask, set_macro, clear_macro) \
    do { if ((data) & (mask)) { set_macro; } else { clear_macro; } } while (0)

// ------------------------- CPU Control pins ---------------------------------
#define CLK_PIN 4
#define RESET_PIN 5

// -------------------------- CPU Input pins ----------------------------------
// We use the analog pins for CPU inputs as they are not 5v tolerant.
#define BHE_PIN 13
#define HOLD_PIN 76 // A0
#define READY_PIN 77 // A1
#define NMI_PIN 78 // A2
#define INTR_PIN 79 // A3
#define READ_BHE_PIN READ_PIN_D13
#define READ_READY_PIN READ_PIN_D77
#define READ_RESET_PIN READ_PIN_D05
#define READ_NMI_PIN READ_PIN_D78
#define READ_INTR_PIN READ_PIN_D79

#define READ_TEST_PIN READ_PIN_A7
#define WRITE_TEST_PIN(x) WRITE_PIN_A7(x) 
#define READ_LOCK_PIN (0) // Currently not connected

#define READ_S0_PIN READ_PIN_D11
#define READ_S1_PIN READ_PIN_D12
#define READ_S2_PIN READ_PIN_D10

// ------------------------- 82288 status pins --------------------------------
#define READ_ALE_PIN READ_PIN_D08

#define READ_MRDC_PIN READ_PIN_D07
#define READ_AMWC_PIN (1)
#define READ_MWTC_PIN READ_PIN_D08
#define READ_IORC_PIN (1)
#define READ_AIOWC_PIN (1)
#define READ_IOWC_PIN (1)

// -------------------------- CPU Bus pins ------------------------------------
#define SET_DBUS_00 do { SET_PIN_D22; } while (0)
#define CLEAR_DBUS_00 do { CLEAR_PIN_D22; } while (0)

#define SET_DBUS_01 do { SET_PIN_D23; } while (0)
#define CLEAR_DBUS_01 do { CLEAR_PIN_D23; } while (0)

#define SET_DBUS_02 do { SET_PIN_D24; } while (0)
#define CLEAR_DBUS_02 do { CLEAR_PIN_D24; } while (0)

#define SET_DBUS_03 do { SET_PIN_D25; } while (0)
#define CLEAR_DBUS_03 do { CLEAR_PIN_D25; } while (0)

#define SET_DBUS_04 do { SET_PIN_D26; } while (0)
#define CLEAR_DBUS_04 do { CLEAR_PIN_D26; } while (0)

#define SET_DBUS_05 do { SET_PIN_D27; } while (0)
#define CLEAR_DBUS_05 do { CLEAR_PIN_D27; } while (0)

#define SET_DBUS_06 do { SET_PIN_D28; } while (0)
#define CLEAR_DBUS_06 do { CLEAR_PIN_D28; } while (0)

#define SET_DBUS_07 do { SET_PIN_D29; } while (0)
#define CLEAR_DBUS_07 do { CLEAR_PIN_D29; } while (0)

#define SET_DBUS_08 do { SET_PIN_D30; } while (0)
#define CLEAR_DBUS_08 do { CLEAR_PIN_D30; } while (0)

#define SET_DBUS_09 do { SET_PIN_D31; } while (0)
#define CLEAR_DBUS_09 do { CLEAR_PIN_D31; } while (0)

#define SET_DBUS_10 do { SET_PIN_D32; } while (0)
#define CLEAR_DBUS_10 do { CLEAR_PIN_D32; } while (0)

#define SET_DBUS_11 do { SET_PIN_D33; } while (0)
#define CLEAR_DBUS_11 do { CLEAR_PIN_D33; } while (0)

#define SET_DBUS_12 do { SET_PIN_D34; } while (0)
#define CLEAR_DBUS_12 do { CLEAR_PIN_D34; } while (0)

#define SET_DBUS_13 do { SET_PIN_D35; } while (0)
#define CLEAR_DBUS_13 do { CLEAR_PIN_D35; } while (0)

#define SET_DBUS_14 do { SET_PIN_D36; } while (0)
#define CLEAR_DBUS_14 do { CLEAR_PIN_D36; } while (0)

#define SET_DBUS_15 do { SET_PIN_D37; } while (0)
#define CLEAR_DBUS_15 do { CLEAR_PIN_D37; } while (0)

#define READ_DBUS_00 READ_PIN_D22
#define READ_DBUS_01 READ_PIN_D23
#define READ_DBUS_02 READ_PIN_D24
#define READ_DBUS_03 READ_PIN_D25
#define READ_DBUS_04 READ_PIN_D26
#define READ_DBUS_05 READ_PIN_D27
#define READ_DBUS_06 READ_PIN_D28
#define READ_DBUS_07 READ_PIN_D29
#define READ_DBUS_08 READ_PIN_D30
#define READ_DBUS_09 READ_PIN_D31
#define READ_DBUS_10 READ_PIN_D32
#define READ_DBUS_11 READ_PIN_D33
#define READ_DBUS_12 READ_PIN_D34
#define READ_DBUS_13 READ_PIN_D35
#define READ_DBUS_14 READ_PIN_D36
#define READ_DBUS_15 READ_PIN_D37

#define READ_ABUS_00 READ_PIN_D38
#define READ_ABUS_01 READ_PIN_D39
#define READ_ABUS_02 READ_PIN_D40
#define READ_ABUS_03 READ_PIN_D41
#define READ_ABUS_04 READ_PIN_D42
#define READ_ABUS_05 READ_PIN_D43
#define READ_ABUS_06 READ_PIN_D44
#define READ_ABUS_07 READ_PIN_D45
#define READ_ABUS_08 READ_PIN_D46
#define READ_ABUS_09 READ_PIN_D47
#define READ_ABUS_10 READ_PIN_D48
#define READ_ABUS_11 READ_PIN_D49
#define READ_ABUS_12 READ_PIN_D50
#define READ_ABUS_13 READ_PIN_D51
#define READ_ABUS_14 READ_PIN_D52
#define READ_ABUS_15 READ_PIN_D53
#define READ_ABUS_16 READ_PIN_D21
#define READ_ABUS_17 READ_PIN_D20 
#define READ_ABUS_18 READ_PIN_D17 // We skip 18 & 19 as they are used for Serial Debugging UART (Serial2)
#define READ_ABUS_19 READ_PIN_D16
#define READ_ABUS_20 READ_PIN_D15
#define READ_ABUS_21 READ_PIN_D14

  // How many cycles to hold the RESET signal high. Intel says "greater than 4" although 4 seems to work.
  #define RESET_HOLD_CYCLE_COUNT 500
  // How many cycles it takes to reset the CPU after RESET signal goes low. First ALE should occur after this many cycles.
  #define RESET_CYCLE_COUNT 500
  // If we didn't see an ALE after this many cycles, give up
  #define RESET_CYCLE_TIMEOUT 2000000
  // What logic level RESET is when asserted
  #define RESET_ASSERT 1
  // What logic level RESET is when deasserted
  #define RESET_DEASSERT 0

class Hat80386 : public HatBase<Hat80386> {

private:
  // Address pins, used for slow address reading via digitalRead()
  static constexpr std::array<int,22> ADDRESS_PINS = {{
    38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, // (16) Bottom half of double-row header
    21, 20, 17, 16, 15, 14
  }};
  const int ADDRESS_LINES = 22;

  static constexpr std::array<int,7> OUTPUT_PINS = {
    4,  // CLK
    5,  // RESET
    76, // HOLD (A0)
    77, // READY (A1)
    78, // NMI (A2)
    79, // INTR (A3)
    83  // TEST/BUSY (A7)
  };

  // All input pins, used to set pin direction on setup
  static constexpr std::array<int,28> INPUT_PINS = {{
    13, 12, 11, 10, 9, 8, // (6) Various signal pins
    38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, // (16) Address pins - Bottom half of double-row header
    21, 20, 17, 16, 15, 14, // (6) Address pins - Top row of GPIO pins
  }};
  
  bool _emulate_bus_controller = false;

protected:
  static constexpr unsigned ClockDivisor = 2;
  size_t addressBusWidth = 22; // Default address bus width is 22 bits
  
public:

  Hat80386(bool emulate_bus_controller = false) : _emulate_bus_controller(emulate_bus_controller) 
    {}

  static void initPins() {
    // Set initial pin modes for all output and input pins
    for (auto pin : OUTPUT_PINS) {
      pinMode(pin, OUTPUT);
    }

    for (auto pin : INPUT_PINS) {
      pinMode(pin, INPUT);
    }

    digitalWrite(INTR_PIN, LOW);  // Must set these to a known value or risk spurious interrupts!
    digitalWrite(NMI_PIN, LOW);   // Must set these to a known value or risk spurious interrupts!
    digitalWrite(HOLD_PIN, LOW);  // Hold pin is active high, make sure it is low by default.
  }

  static void tickCpuImpl() {
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
  uint32_t readAddressBus(bool peek) {
    // If we're not peeking, set the bus direction to input
    if (!peek) {
      setBusDirection(BusDirection::Input, ActiveBusWidth::Sixteen);
    }

    uint32_t address = 0;
    // Read the address bus pins
    if (READ_ABUS_00) address |= 0x00000001;
    if (READ_ABUS_01) address |= 0x00000002;
    if (READ_ABUS_02) address |= 0x00000004;
    if (READ_ABUS_03) address |= 0x00000008;
    if (READ_ABUS_04) address |= 0x00000010;
    if (READ_ABUS_05) address |= 0x00000020;
    if (READ_ABUS_06) address |= 0x00000040;
    if (READ_ABUS_07) address |= 0x00000080;
    if (READ_ABUS_08) address |= 0x00000100;
    if (READ_ABUS_09) address |= 0x00000200;
    if (READ_ABUS_10) address |= 0x00000400;
    if (READ_ABUS_11) address |= 0x00000800;
    if (READ_ABUS_12) address |= 0x00001000;
    if (READ_ABUS_13) address |= 0x00002000;
    if (READ_ABUS_14) address |= 0x00004000;
    if (READ_ABUS_15) address |= 0x00008000;
    if (READ_ABUS_16) address |= 0x00010000;
    if (READ_ABUS_17) address |= 0x00020000;
    if (READ_ABUS_18) address |= 0x00040000;
    if (READ_ABUS_19) address |= 0x00080000;
    if (READ_ABUS_20) address |= 0x00100000;
    if (READ_ABUS_21) address |= 0x00200000;
    //if (READ_ABUS_22) address |= 0x00400000;
    //if (READ_ABUS_23) address |= 0x00800000;
    return address;
  }

  static void writePinImpl(OutputPin pin, bool value) {
    switch (pin) {
      case OutputPin::Ready:
        WRITE_PIN_A1(value);
        break;
      case OutputPin::Test:
        // !BUSY is tied to Vcc, so we don't control it.
        //WRITE_PIN_D05(value);
        break;
      case OutputPin::Intr:
        WRITE_PIN_A3(value);
        break;
      case OutputPin::Nmi:
        WRITE_PIN_A2(value);
        break;
      default:
        // Handle other pins if necessary
        break;
    }
  }

  template<typename Board>
  static CpuResetResult resetCpuImpl(Board& board) {

    CpuResetResult result = {};
    result.success = false;
    result.queueStatus = false;
    result.busWidth = BusWidth::Sixteen; // 286 is always 16-bit bus width

    digitalWrite(INTR_PIN, LOW); // INTR must be low or CPU will immediately interrupt.
    digitalWrite(NMI_PIN, LOW); // NMI must be low or CPU will immediately interrupt.
    digitalWrite(HOLD_PIN, LOW); // HOLD must be low or CPU will not reset.

    bool ale_went_off = false;
    //bool bhe_went_off = false;

    WRITE_TEST_PIN(1); // Set TEST pin high to allow CPU to reset without entering test mode.
    delayMicroseconds(10); // Make sure that TEST pin is settled high before asserting RESET.
    WRITE_PIN_D05(0); // Assert RESET high for hold count.

    for (int i = 0; i < RESET_HOLD_CYCLE_COUNT; i++) {
      if (READ_ALE_PIN == false) {
        ale_went_off = true;
      }
      //tickCpu();
      cycle();
    }

    // CPU didn't reset for some reason.
    if (ale_went_off == false) {
      //set_error("CPU failed to reset: ALE not off!");   
      return result;
    }

    // Deassert RESET pin to allow CPU to start.
    WRITE_PIN_D05(1);

    // Clock CPU while waiting for ALE
    int ale_cycles = 0;

    // Reset takes 7 cycles, bit we can try for longer
    for ( int i = 0; i < RESET_CYCLE_TIMEOUT; i++ ) {
      cycle();

      if (!READ_ALE_PIN) {
        if (!ale_went_off) {
          ale_went_off = true;
        }
      }

      // if (!READ_BHE_PIN) {
      //     bhe_went_off = true;
      // }

      ale_cycles++;      

      if (ale_went_off && READ_ALE_PIN) {
        // ALE is active! CPU has successfully reset
        //CPU.doing_reset = false;      
  
        board.debugPrintln(DebugType::RESET, "###########################################");      
        board.debugPrintln(DebugType::RESET, "## Reset CPU!                            ##");
        board.debugPrintln(DebugType::RESET, "###########################################");
        result.success = true;
      }
    }

    if (!result.success) {
      // ALE did not turn on within the specified cycle timeout, so we failed to reset the cpu.
      #if DEBUG_RESET
        board.debugPrintln(DebugType::ERROR, "## Failed to reset CPU! ##");
      #endif
      //set_error("CPU failed to reset: No ALE!");   
    }
    
    
    return result;
  }

  static bool readBHEPinImpl() {
    // Read the BHE pin (Bus High Enable)
    return READ_BHE_PIN;
  }
  static bool readALEPinImpl() {
    return READ_ALE_PIN;
  }
  static bool readMRDCPinImpl() {
    return READ_MRDC_PIN;
  }
  static bool readAMWCPinImpl() {
    return READ_AMWC_PIN;
  }
  static bool readMWTCPinImpl() {
    return READ_MWTC_PIN;
  }
  static bool readIORCPinImpl() {
    return READ_IORC_PIN;
  }
  static bool readIOWCPinImpl() {
    return READ_IOWC_PIN;
  }
  static bool readAIOWCPinImpl() {
    return READ_AIOWC_PIN;
  }
  static bool readINTAPinImpl() {
    return false;
  }
  static void writeResetPinImpl(bool value) {
    // Write to the RESET pin
    WRITE_PIN_D05(value);
  }

  static uint8_t readCpuStatusLinesImpl() {
    // Read the CPU status lines
    uint8_t status = 0;
    if (READ_S0_PIN) { status |= 0x01; }; // S0
    if (READ_S1_PIN) { status |= 0x02; }; // S1
    if (READ_S2_PIN) { status |= 0x04; }; // S2
    
    //if (READ_PIN_D38) { status |= 0x08; }; // S3
    //if (READ_PIN_D39) { status |= 0x10; }; // S4
    //if (READ_PIN_D40) { status |= 0x20; }; // S5
    
    // Queue status lines not supported on 80286
    //if (READ_PIN_D09) { status |= 0x40; }; // QS0
    //if (READ_PIN_D08) { status |= 0x80; }; // QS1

    return status;
  }

  static uint8_t readBusControllerCommandLinesImpl() {
    // Read the bus controller command lines
    uint8_t command = 0;
    command |= readMRDCPinImpl() ? 0x01 : 0;     // MRDC - Pin 51
    command |= readAMWCPinImpl() ? 0x02 : 0;     // AMWC - Pin 52
    command |= readMWTCPinImpl() ? 0x04 : 0;     // MWTC - Pin 53
    command |= readIORCPinImpl() ? 0x08 : 0;     // IORC - Pin 46
    command |= readAIOWCPinImpl() ? 0x10 : 0;    // AIOWC- Pin 48
    command |= readIOWCPinImpl() ? 0x20 : 0;     // IOWC - Pin 47
    command |= readINTAPinImpl() ? 0x40 : 0;     // INTA - Pin 45
    // Although not an 8288 command status, we have an extra bit, so we can stick BHE in here.
    // This saves us from needing to add an extra byte - that adds up!
    command |= READ_BHE_PIN ? 0x80 : 0;
    return command;
  }

  static uint8_t readBusControllerControlLinesImpl() {
    // Read the bus controller control lines
    uint8_t control = 0;
    control |= READ_ALE_PIN ? 0x01 : 0;     // ALE      - Pin 50
    //control |= READ_PIN_D49 ? 0x02 : 0;     // DTR      - Pin 49
    //control |= READ_PIN_D43 ? 0x04 : 0;     // MCE/PDEN - Pin 43
    //control |= READ_PIN_D44 ? 0x08 : 0;     // DEN      - Pin 44
    return control;
  }
};