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
#include <hats/HatBase.h>
#include <hats/HatBase.h>
#include <hats/Pins.h>
#include <DebugFilter.h>

#define CPU_8088
#define USE_SETUP_PROGRAM 0
#define SETUP_PROGRAM SETUP_PROGRAM_86
#define SETUP_PROGRAM_PATCH_OFFSET 0

#define STORE_IO_BASE 0x0000

#define PRE_RESET_CYCLE_COUNT 5 // How many cycles to wait before asserting RESET. This gives time for pins to settle.
// How many cycles to hold the RESET signal high. Intel says 18 cycles for the 80286. 
#define RESET_HOLD_CYCLE_COUNT 8
// How many cycles it takes to reset the CPU after RESET signal goes low. Intel says 38 cycles for the 80286. 
// If we didn't see an ALE after this many cycles, give up
#define RESET_CYCLE_TIMEOUT 40
// What logic level RESET is when asserted
#define RESET_ASSERT 1
// What logic level RESET is when deasserted
#define RESET_DEASSERT 0

#define ADDRESS_SPACE_MASK 0xFFFFF // 20-bit address space for 8088
#define WRITE_CYCLE T3
#define STORE_TIMEOUT 1000
#define LOAD_TIMEOUT 1000

#define WRITE_BIT(data, mask, set_macro, clear_macro) \
    do { if ((data) & (mask)) { set_macro; } else { clear_macro; } } while (0)

// Data bus mappings for 8088 hat.
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

#define READ_ABUS_00 READ_PIN_D22
#define READ_ABUS_01 READ_PIN_D23
#define READ_ABUS_02 READ_PIN_D24
#define READ_ABUS_03 READ_PIN_D25
#define READ_ABUS_04 READ_PIN_D26
#define READ_ABUS_05 READ_PIN_D27
#define READ_ABUS_06 READ_PIN_D28
#define READ_ABUS_07 READ_PIN_D29
#define READ_ABUS_08 READ_PIN_D30
#define READ_ABUS_09 READ_PIN_D31
#define READ_ABUS_10 READ_PIN_D32
#define READ_ABUS_11 READ_PIN_D33
#define READ_ABUS_12 READ_PIN_D34
#define READ_ABUS_13 READ_PIN_D35
#define READ_ABUS_14 READ_PIN_D36
#define READ_ABUS_15 READ_PIN_D37
#define READ_ABUS_16 READ_PIN_D38
#define READ_ABUS_17 READ_PIN_D39
#define READ_ABUS_18 READ_PIN_D40
#define READ_ABUS_19 READ_PIN_D41

#if defined(__SAM3X8E__) // If Arduino DUE
  #define SET_DATA_BUS_TO_READ do { \
    uint32_t pins_b = BIT26; \
    uint32_t pins_a = BIT07 | BIT14 | BIT15; \
    uint32_t pins_c = 0x01FF; \
    uint32_t pins_d = BIT00 | BIT01 | BIT02 | BIT03 | BIT06 | BIT09 | BIT10; \
    PIOA->PIO_ODR = pins_a; \
    PIOB->PIO_ODR = pins_b; \
    PIOC->PIO_ODR = pins_c; \
    PIOD->PIO_ODR = pins_d; \
    delayMicroseconds(PIN_CHANGE_DELAY); \
  } while (0)
#elif defined(ARDUINO_GIGA) // If Arduino GIGA
  #define PORT_K_DBUS_MASK 0xFFFFC03F
  #define PORT_K_DBUS_READ 0
  #define PORT_K_DBUS_WRITE 0x00001540
  #define PORT_J_DBUS_MASK 0x0CFFC000
  #define PORT_J_DBUS_READ 0
  #define PORT_J_DBUS_WRITE 0x51001555
  #define PORT_J_DBUS_WRITE_HI 0x00001540
  #define PORT_J_DBUS_WRITE_LO 0x51000015
  #define PORT_G_DBUS_MASK 0xF0FFFFFF
  #define PORT_G_DBUS_READ 0
  #define PORT_G_DBUS_WRITE 0x05000000

  #define SET_DATA_BUS_TO_READ do { \
    GPIOK->MODER = (GPIOK->MODER & PORT_K_DBUS_MASK) | PORT_K_DBUS_READ; \
    GPIOJ->MODER = (GPIOJ->MODER & PORT_J_DBUS_MASK) | PORT_J_DBUS_READ; \
    GPIOG->MODER = (GPIOG->MODER & PORT_G_DBUS_MASK) | PORT_G_DBUS_READ; \
    PIN_CHANGE_DELAY; \
  } while (0)

  #define SET_DATA_BUS_TO_WRITE_LO do { \
    GPIOJ->MODER = (GPIOJ->MODER & PORT_J_DBUS_MASK) | PORT_J_DBUS_WRITE_LO; \
    GPIOG->MODER = (GPIOG->MODER & PORT_G_DBUS_MASK) | PORT_G_DBUS_WRITE; \
    PIN_CHANGE_DELAY; \
  } while (0)

  #define SET_DATA_BUS_TO_WRITE_HI do { \
    GPIOK->MODER = (GPIOK->MODER & PORT_K_DBUS_MASK) | PORT_K_DBUS_WRITE; \
    GPIOJ->MODER = (GPIOJ->MODER & PORT_J_DBUS_MASK) | PORT_J_DBUS_WRITE_HI; \
    PIN_CHANGE_DELAY; \
  } while (0)

  #define SET_DATA_BUS_TO_WRITE do { \
    GPIOK->MODER = (GPIOK->MODER & PORT_K_DBUS_MASK) | PORT_K_DBUS_WRITE; \
    GPIOJ->MODER = (GPIOJ->MODER & PORT_J_DBUS_MASK) | PORT_J_DBUS_WRITE; \
    GPIOG->MODER = (GPIOG->MODER & PORT_G_DBUS_MASK) | PORT_G_DBUS_WRITE; \
    PIN_CHANGE_DELAY; \
  } while (0)
#endif


// -----------------------------Buzzer ----------------------------------------
#define BUZZER_PIN 2

#if defined(__AVR_ATmega2560__) // If Arduino MEGA

  #define WRITE_BUZZER(x) ((x) ? (PORTE |= (1 << 4)) : (PORTE &= ~(1 << 4)))

#elif defined(__SAM3X8E__) // If Arduino DUE

  #define WRITE_BUZZER(x) ((x) ? (PIOB->PIO_SODR = PIO_PB25) : (PIOB->PIO_CODR = PIO_PB25))

#elif defined(ARDUINO_GIGA)

  // do buzzer here
  #define WRITE_BUZZER(x) ((X))
#endif

// ------------------------- CPU Control pins ---------------------------------

#define CLK_PIN 4
#define RESET_PIN 5

// -------------------------- CPU Input pins ----------------------------------
#define BHE_PIN 17
#define S0_PIN 14
#define S1_PIN 15
#define S2_PIN 16
#define READ_BHE_PIN READ_PIN_D17
#define READ_RESET_PIN READ_PIN_D05
#define READ_READY_PIN READ_PIN_D06
#define READ_TEST_PIN READ_PIN_D07
#define READ_INTR_PIN READ_PIN_D12
#define READ_NMI_PIN READ_PIN_D13
#define READ_S0_PIN READ_PIN_D14
#define READ_S1_PIN READ_PIN_D15
#define READ_S2_PIN READ_PIN_D16
#define READ_S3_PIN READ_PIN_D38
#define READ_S4_PIN READ_PIN_D39
#define READ_S5_PIN READ_PIN_D40
#define READ_QS0_PIN READ_PIN_D09
#define READ_QS1_PIN READ_PIN_D08

#define READY_PIN 6
#define TEST_PIN 7
#define LOCK_PIN 10
#define INTR_PIN 12
#define NMI_PIN 13

// -------------------------- CPU Output pins ---------------------------------
#define RQ_PIN 3

// --------------------------8288 Control Inputs ------------------------------
// These are mapped to analog pins A0 & A1 which map to different digital pin
// numbers on different boards.
#if defined (ARDUINO_GIGA)
#define AEN_PIN 76
#define CEN_PIN 77
#else
#define AEN_PIN 54
#define CEN_PIN 55
#endif
// --------------------------8288 Control lines -------------------------------
#define ALE_PIN 50
#define DTR_PIN 49
#define MCEPDEN_PIN 43
#define DEN_PIN 44

// --------------------------8288 Command lines -------------------------------
#define MRDC_PIN 51
#define AMWC_PIN 52
#define MWTC_PIN 53
#define IORC_PIN 46
#define AIOWC_PIN 48
#define IOWC_PIN 47
#define INTA_PIN 45

#define READ_ALE_PIN      READ_PIN_D50
#define READ_MRDC_PIN     READ_PIN_D51
#define READ_AMWC_PIN     READ_PIN_D52
#define READ_MWTC_PIN     READ_PIN_D53
#define READ_IORC_PIN     READ_PIN_D46
#define READ_AIOWC_PIN    READ_PIN_D48
#define READ_IOWC_PIN     READ_PIN_D47
#define READ_INTA_PIN     READ_PIN_D45

// -------------------------- Macro definitions  ---------------------------------

// Write macros
#if defined(__SAM3X8E__) // If Arduino DUE
  // D4: PC26* (some references say PA29 - didn't work)
  #define WRITE_CLK(x) ((x) ? (PIOC->PIO_SODR = BIT26) : (PIOC->PIO_CODR = BIT26))
  // D5: PC25
  #define WRITE_RESET(x) ((x) ? (PIOC->PIO_SODR = PIO_PC25) : (PIOC->PIO_CODR = PIO_PC25))
  // D6: PC24
  #define WRITE_READY_PIN(x) ((x) ? (PIOC->PIO_SODR = BIT24) : (PIOC->PIO_CODR = BIT24))
  // D7: PC23
  #define WRITE_TEST_PIN(x) ((x) ? (PIOC->PIO_SODR = BIT23) : (PIOC->PIO_CODR = BIT23))
  // D10: PC29*
  #define WRITE_LOCK_PIN(x) ((x) ? (PIOC->PIO_SODR = BIT29) : (PIOC->PIO_CODR = BIT29))
  // D12: PD8
  #define WRITE_INTR_PIN(x) ((x) ? (PIOD->PIO_SODR = BIT08) : (PIOD->PIO_CODR = BIT08))
  // D13: PB27
  #define WRITE_NMI_PIN(x) ((x) ? (PIOB->PIO_SODR = BIT27) : (PIOB->PIO_CODR = BIT27))
  // A0: PA16
  #define WRITE_AEN_PIN(x) ((x) ? (PIOA->PIO_SODR = BIT16) : (PIOA->PIO_CODR = BIT16))
  // A1: PA24
  #define WRITE_CEN_PIN(x) ((x) ? (PIOA->PIO_SODR = BIT24) : (PIOA->PIO_CODR = BIT24))

#elif defined(__AVR_ATmega2560__) // If Arduino MEGA
  // D4
  #define WRITE_CLK(x) ((x) ? (PORTG |= (1 << 5)) : (PORTG &= ~(1 << 5))) // CLK is PG5
  // D5
  #define WRITE_RESET(x) ((x) ? (PORTE |= (1 << 3)) : (PORTE &= ~(1 << 3))) // RESET is PE3
  // D6
  #define WRITE_READY_PIN(x) ((x) ? (PORTH |= (1 << 3)) : (PORTH &= ~(1 << 3)))
  // D7
  #define WRITE_TEST_PIN(x) ((x) ? (PORTH |= (1 << 4)) : (PORTH &= ~(1 << 4)))
  // D10
  #define WRITE_LOCK_PIN(x) ((x) ? (PORTB |= (1 << 4)) : (PORTB &= ~(1 << 4)))
  // D12
  #define WRITE_INTR_PIN(x) ((x) ? (PORTB |= (1 << 6)) : (PORTB &= ~(1 << 6)))
  // D13
  #define WRITE_NMI_PIN(x) ((x) ? (PORTB |= (1 << 7)) : (PORTB &= ~(1 << 7)))
  // A0
  #define WRITE_AEN_PIN(x) ((x) ? (PORTF |= 0x01) : (PORTF &= ~0x01))
  // A1
  #define WRITE_CEN_PIN(x) ((x) ? (PORTF |= (1 << 1)) : (PORTF &= ~(1 << 1)))

#elif defined (ARDUINO_GIGA)

  // D4: PJ8
  #define WRITE_CLK(x) WRITE_PIN_D04(x)
  // D5: PA7
  #define WRITE_RESET(x) WRITE_PIN_D05(x)
  // D6: PD13
  #define WRITE_READY_PIN(x) WRITE_PIN_D06(x)
  // D7: PB4
  #define WRITE_TEST_PIN(x) WRITE_PIN_D07(x)
  // D10: PK1
  #define WRITE_LOCK_PIN(x) WRITE_PIN_D10(x)
  // D12: PJ11
  #define WRITE_INTR_PIN(x) WRITE_PIN_D12(x)
  // D13: PH6
  #define WRITE_NMI_PIN(x) WRITE_PIN_D13(x)
  // A0: PC4
  #define WRITE_AEN_PIN(x) WRITE_PIN_A0(x)
  // A1: PC5
  #define WRITE_CEN_PIN(x) WRITE_PIN_A1(x)

#endif

// Read macros

#if defined(__SAM3X8E__) // If Arduino DUE
  #define READ_LOCK_PIN READ_PIN_D10
#elif defined(__AVR_ATmega2560__) // If Arduino MEGA
  #define READ_LOCK_PIN 0
#elif defined(ARDUINO_GIGA)
  #define READ_LOCK_PIN READ_PIN_D10
#endif

class Hat8088 : public HatBase<Hat8088> {

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

protected:

public:

  BusDirection busDirection = BusDirection::Input; // Default bus direction is input
  size_t dataBusWidth = 16; // Default data bus width is 16 bits
  size_t addressBusWidth = 22; // Default address bus width is 22 bits

  static void initPins() {
    // Set initial pin modes for all output and input pins
    for (auto pin : OUTPUT_PINS) {
      pinMode(pin, OUTPUT);
    }

    for (auto pin : INPUT_PINS) {
      pinMode(pin, INPUT);
    }

#if defined(ARDUINO_GIGA)
    // Set S0-S2 pin pullups
    pinMode(S0_PIN, INPUT_PULLUP);
    pinMode(S1_PIN, INPUT_PULLUP);
    pinMode(S2_PIN, INPUT_PULLUP);
#endif

#ifdef HAT_8087_V1
      pinMode(RQ_PIN, INPUT);
#else
      digitalWrite(RQ_PIN, HIGH);  // Don't allow other bus masters
#endif

    // Default output pin states
    digitalWrite(READY_PIN, HIGH);
    digitalWrite(INTR_PIN, LOW);  // Must set these to a known value or risk spurious interrupts!
    digitalWrite(NMI_PIN, LOW);   // Must set these to a known value or risk spurious interrupts!
    // Enable i8228 outputs
    digitalWrite(AEN_PIN, LOW);   // AEN is enable-low
    digitalWrite(CEN_PIN, HIGH);  // Command enable enables the outputs on the i8288
  }

  template<typename Board>
  CpuResetResult resetCpuImpl(Board& board) {
    CpuResetResult result = {};
    result.success = false;
    result.queueStatus = false;
    result.busWidth = BusWidth::Eight;
    setBusDirection(BusDirection::Input, ActiveBusWidth::Sixteen);
    WRITE_TEST_PIN(0);
    WRITE_INTR_PIN(0);
    WRITE_NMI_PIN(0);
    WRITE_RESET(0);

    bool ale_went_off = false;
    bool bhe_went_off = false;
    bool qs0_high = false;
    bool have_i8288 = false;
    
    // Clock the CPU a few times before asserting RESET.
    for (int i = 0; i < PRE_RESET_CYCLE_COUNT; i++) {
      tickCpu();
    }

    // Enable i8288 outputs.
    digitalWrite(AEN_PIN, LOW);
    digitalWrite(CEN_PIN, HIGH);

    // Assert RESET high for hold count.
    WRITE_RESET(1);

    for (int i = 0; i < RESET_HOLD_CYCLE_COUNT; i++) {
      if (READ_ALE_PIN == false) {
        ale_went_off = true;
      }
      cycle();
    }


    // Check the i8288 control lines
    if (!READ_MRDC_PIN && !READ_MWTC_PIN && !READ_AMWC_PIN) {
      board.debugPrintln(DebugType::RESET, "## No i8288 detected ##");
      have_i8288 = false;
    }
    else {
      board.debugPrintln(DebugType::RESET, "## i8288 detected ##");
      have_i8288 = true;
    }

    WRITE_RESET(0);

    // Clock CPU while waiting for ALE
    int ale_cycles = 0;

    // Reset takes 7 cycles, bit we can try for longer
    for (int i = 0; i < RESET_CYCLE_TIMEOUT; i++) {
      cycle();

      if (READ_QS0_PIN) {
        qs0_high = true;
      }

      if (!READ_ALE_PIN) {
        if (!ale_went_off) {
          ale_went_off = true;
        }
      }

      if (!READ_BHE_PIN) {
          bhe_went_off = true;
      }

      ale_cycles++;      

      if (ale_went_off && READ_ALE_PIN) {
        // ALE is active! CPU has successfully reset
        board.debugPrintln(DebugType::RESET, "###########################################");
        if (bhe_went_off) {
          board.debugPrintln(DebugType::RESET, "## Reset CPU: 16-bit bus detected        ##");
          result.busWidth = BusWidth::Sixteen;
        }
        else {
          board.debugPrintln(DebugType::RESET, "## Reset CPU:  8-bit bus detected        ##");
          result.busWidth = BusWidth::Eight;
        }
        if (qs0_high) {
          board.debugPrintln(DebugType::RESET, "## Queue status lines appear unavailable ##");
        }
        board.debugPrintln(DebugType::RESET, "###########################################");
        result.success = true;
        result.queueStatus = !qs0_high;
        break;
      }
    }
    return result;
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
  }

  /// @brief Return true if the current hat has a multiplexed bus.
  static bool hasMultiplexedBusImpl() {
    return true;
  }

  static bool isTransferDoneImpl(BusStatus latched_status) {
    switch (latched_status) {
      case IOR:
        // IORC is active-low, so we are returning true if it is off
        return READ_IORC_PIN;
      case IOW:
        // IOWC is active-low, so we are returning true if it is off
        return READ_IOWC_PIN;
      case CODE:
        // FALLTHRU
      case MEMR:
        // MRDC is active-low, so we are returning true if it is off
        return READ_MRDC_PIN;
      case MEMW:
        // MWTC is active-low, so we are returning true if it is off
        return READ_MWTC_PIN;
      default:
        // Rely on external READY pin
        return READ_READY_PIN;
        break;
    }
  }

  static TCycle getNextCycleImpl(TCycle current_cycle, BusStatus current_status, BusStatus latched_status) {
    // Return the next cycle for the CPU
     switch (current_cycle) {
      case TI:
        break;
      
      case T1:
        // Begin a bus cycle only if signalled, otherwise wait in T1
        if (current_status != PASV) {
          return T2;
        }
        break;

      case T2:
        return T3;
        break;

      case T3:
        if (isTransferDoneImpl(latched_status)) {
          return T4;
        } else {
  #if DEBUG_TSTATE
          debugPrintlnColor(ansi::yellow, "Setting T-cycle to Tw");
  #endif
          return TW;
        }
        break;

      case TW:
        // Transition to T4 if read/write signals are complete
        if (isTransferDoneImpl(latched_status)) {
          return T4;
        }
        break;

      case T4:
  #if DEBUG_TSTATE
        debugPrintlnColor(ansi::yellow, "Setting T-cycle to T1");
  #endif
        return T1;
        break;
    }

    return current_cycle; // If no change, return the current cycle
  }

  uint16_t readDataBus(ActiveBusWidth width, bool peek = false) {
    if(!peek) {
      setBusDirection(BusDirection::Input, width);
    };
    uint16_t data = 0;
    if (peek || (width == ActiveBusWidth::EightLow) || (width == ActiveBusWidth::Sixteen)) {
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
    if (peek || (width == ActiveBusWidth::EightHigh) || (width == ActiveBusWidth::Sixteen)) {
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
        DEBUG_SERIAL.print("Writing low data bus: ");
        DEBUG_SERIAL.println(data & 0x00FF, HEX);
        WRITE_BIT(data, 0x01, SET_DBUS_00, CLEAR_DBUS_00);
        WRITE_BIT(data, 0x02, SET_DBUS_01, CLEAR_DBUS_01);
        WRITE_BIT(data, 0x04, SET_DBUS_02, CLEAR_DBUS_02);
        WRITE_BIT(data, 0x08, SET_DBUS_03, CLEAR_DBUS_03);
        WRITE_BIT(data, 0x10, SET_DBUS_04, CLEAR_DBUS_04);
        WRITE_BIT(data, 0x20, SET_DBUS_05, CLEAR_DBUS_05);
        WRITE_BIT(data, 0x40, SET_DBUS_06, CLEAR_DBUS_06);
        WRITE_BIT(data, 0x80, SET_DBUS_07, CLEAR_DBUS_07);

        // SET_DBUS_00;
        // SET_DBUS_01;
        // SET_DBUS_02;
        // SET_DBUS_03;
        // SET_DBUS_04;
        // SET_DBUS_05;
        // SET_DBUS_06;
        // SET_DBUS_07;

        // CLEAR_DBUS_00;
        // CLEAR_DBUS_01;
        // CLEAR_DBUS_02;
        // CLEAR_DBUS_03;
        // CLEAR_DBUS_04;
        // CLEAR_DBUS_05;
        // CLEAR_DBUS_06;
        // CLEAR_DBUS_07;

      }
      
      if ((width == ActiveBusWidth::EightHigh) || (width == ActiveBusWidth::Sixteen)) {
        DEBUG_SERIAL.print("Writing high data bus: ");
        DEBUG_SERIAL.println(data >> 8, HEX);
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
  uint32_t readAddressBusImpl(bool peek = true) {
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

    static bool readBHEPinImpl() {
    // Read the BHE pin (Bus High Enable)
    return READ_BHE_PIN;
  }
  static bool readALEPinImpl() {
    return READ_ALE_PIN;
  }
  static bool readLockPinImpl() {
    return READ_LOCK_PIN;
  }
  static bool readReadyPinImpl() {
    return READ_READY_PIN;
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

  static void writePinImpl(OutputPin pin, bool value) {
    switch (pin) {
      case OutputPin::Ready:
        WRITE_READY_PIN(value);
        break;
      case OutputPin::Test:
        WRITE_TEST_PIN(value);
        break;
      case OutputPin::Intr:
        WRITE_INTR_PIN(value);
        break;
      case OutputPin::Nmi:
        WRITE_NMI_PIN(value);
        break;
      default:
        // Handle other pins if necessary
        break;
    }
  }

  static uint8_t readCpuStatusLinesImpl() {
    // Read the CPU status lines
    uint8_t status = 0;
    if (READ_S0_PIN) { status |= 0x01; };
    if (READ_S1_PIN) { status |= 0x02; };
    if (READ_S2_PIN) { status |= 0x04; };
    if (READ_S3_PIN) { status |= 0x08; };
    if (READ_S4_PIN) { status |= 0x10; };
    if (READ_S5_PIN) { status |= 0x20; };
    if (READ_QS0_PIN) { status |= 0x40; };
    if (READ_QS1_PIN) { status |= 0x80; };
    return status;
  }

  static uint8_t readBusControllerCommandLinesImpl() {
    // Read the bus controller command lines
    uint8_t command = 0;
    command |= READ_MRDC_PIN ? 0x01 : 0;     // MRDC - Pin 51
    command |= READ_AMWC_PIN ? 0x02 : 0;     // AMWC - Pin 52
    command |= READ_MWTC_PIN ? 0x04 : 0;     // MWTC - Pin 53
    command |= READ_IORC_PIN ? 0x08 : 0;     // IORC - Pin 46
    command |= READ_AIOWC_PIN ? 0x10 : 0;    // AIOWC- Pin 48
    command |= READ_IOWC_PIN ? 0x20 : 0;     // IOWC - Pin 47
    command |= READ_INTA_PIN ? 0x40 : 0;     // INTA - Pin 45
    // Although not an 8288 command status, we have an extra bit, so we can stick BHE in here.
    command |= READ_BHE_PIN ? 0x80 : 0;      // BHE  - Pin 17
    return command;
  }

  static uint8_t readBusControllerControlLinesImpl() {
    // Read the bus controller control lines
    uint8_t control = 0;
    control |= READ_ALE_PIN ? 0x01 : 0;     // ALE      - Pin 50
    control |= READ_PIN_D49 ? 0x02 : 0;     // DTR      - Pin 49
    control |= READ_PIN_D43 ? 0x04 : 0;     // MCE/PDEN - Pin 43
    control |= READ_PIN_D44 ? 0x08 : 0;     // DEN      - Pin 44
    return control;
  }
};