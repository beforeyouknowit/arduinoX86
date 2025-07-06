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

// Base class for the various ArduinoX86 hats. It is assumed that most hats will
// use D22-D37 for the data bus, so default implementations are provided for 
// working with the data bus.

#include <Arduino.h>
#include <BusTypes.h>
#include <CpuTypes.h>
#include <gpio_pins.h>
#include <hats/Pins.h>
#include <ansi_color.h>

template<typename Derived>
class HatBase {

  public: 
    void setBusDirection(BusDirection direction, ActiveBusWidth width = ActiveBusWidth::Sixteen) {
      // dispatch to Derived::setBusDirectionImpl(bool)
      static_cast<Derived*>(this)->setBusDirectionImpl(direction, width);
    }

    template<typename Board>
    CpuResetResult resetCpu(Board& board) {
      // dispatch to Derived::cpuResetImpl()
      return static_cast<Derived*>(this)->resetCpuImpl(board);
    }

    void tickCpu() {
      static_cast<Derived*>(this)->tickCpuImpl();
    };

    static void writePin(OutputPin pin, bool value) {
      Derived::writePinImpl(pin, value);
    }

    template<typename Board>
    uint16_t readDataBus( Board& board, ActiveBusWidth bus_width, bool peek = false) {
      return static_cast<Derived*>(this)->readDataBusImpl(board, bus_width, peek);
    }

    template<typename Board>
    void writeDataBus( Board& b, uint16_t d, ActiveBusWidth w ) {
      static_cast<Derived*>(this)->writeDataBusImpl(b, d, w);
    }

    static uint32_t readAddressBus(bool peek) {
       return Derived::readAddressBusImpl(peek);
    }

    int8_t readCpuStatusLines() {
       return static_cast<Derived*>(this)->readCpuStatusLinesImpl();
    }

    static bool cpuIsReading(BusTransferType &read_type) {
       return Derived::cpuIsReadingImpl(read_type);
    }

    static bool cpuIsWriting(BusTransferType &write_type) {
       return Derived::cpuIsWritingImpl(write_type);
    }

    static bool readBHEPin()   { return Derived::readBHEPinImpl(); }

    // The following methods are not static as they may need to access state for bus controller emulation.
    bool readALEPin()   { return static_cast<Derived*>(this)->readALEPinImpl(); }
    bool readMRDCPin()  { return static_cast<Derived*>(this)->readMRDCPinImpl(); }
    bool readAMWCPin()  { return static_cast<Derived*>(this)->readAMWCPinImpl(); }
    bool readMWTCPin()  { return static_cast<Derived*>(this)->readMWTCPinImpl(); }
    bool readIORCPin()  { return static_cast<Derived*>(this)->readIORCPinImpl(); }
    bool readIOWCPin()  { return static_cast<Derived*>(this)->readIOWCPinImpl(); }
    bool readAIOWCPin() { return static_cast<Derived*>(this)->readAIOWCPinImpl(); }

    static void writeResetPin(bool value) {
       Derived::writeResetPinImpl(value);
    }

    uint8_t readBusControllerCommandLines() {
       return static_cast<Derived*>(this)->readBusControllerCommandLinesImpl();
    }

    uint8_t readBusControllerControlLines() {
       return static_cast<Derived*>(this)->readBusControllerControlLinesImpl();
    }

    static int getAddressBusWidth() {
      // Default address bus width is 20 bits
      return 20;
    }

    static TCycle getNextCycle(TCycle current_cycle, BusStatus current_status, BusStatus latched_status) {
      return Derived::getNextCycleImpl(current_cycle, current_status, latched_status);
    }

    static bool isTransferDone(BusStatus latched_status) {
      return Derived::isTransferDoneImpl(latched_status);
    }

    static const char *getTCycleString(TCycle cycle) {
      switch (cycle) {
        case T1: return "T1";
        case T2: return "T2";
        case T3: return "T3";
        case T4: return "T4";
        case TW: return "Tw";
        case TI: return "Ti";
        default: return "T?";
      }
    }

    static BusStatus decodeBusStatus(uint8_t status_byte ) {
      switch (status_byte & 0x07) {
        case 0: return BusStatus::IRQA;
        case 1: return BusStatus::IOR;
        case 2: return BusStatus::IOW;
        case 3: return BusStatus::HALT;
        case 4: return BusStatus::CODE;
        case 5: return BusStatus::MEMR;
        case 6: return BusStatus::MEMW;
        case 7: return BusStatus::PASV;
        default: return BusStatus::PASV;
      }
    }

    static const char *getBusStatusString(BusStatus status) {
      switch (status) {
        case BusStatus::IRQA: return "IRQA";
        case BusStatus::IOR:  return "IOR ";
        case BusStatus::IOW:  return "IOW ";
        case BusStatus::HALT: return "HALT";
        case BusStatus::CODE: return "CODE";
        case BusStatus::MEMR: return "MEMR";
        case BusStatus::MEMW: return "MEMW";
        case BusStatus::PASV: return "PASV";
        default: return "PASV";
      }
    }

    static const char *getBusStatusColor(BusStatus status) {
      switch (status) {
        case BusStatus::IRQA: return ansi::bright_red;
        case BusStatus::IOR:  return ansi::yellow;
        case BusStatus::IOW:  return ansi::bright_yellow;
        case BusStatus::HALT: return ansi::bright_magenta;
        case BusStatus::CODE: return ansi::cyan;
        case BusStatus::MEMR: return ansi::bright_blue;
        case BusStatus::MEMW: return ansi::bright_green;
        case BusStatus::PASV: return ansi::white;
        default: return ansi::white;
      }
    }

  /// @brief  Return true if the current hat has segment status lines.
  static bool hasSegmentStatus() {
    return true;
  }

  /// @brief Return true if the current hat has a multiplexed bus.
  static bool hasMultiplexedBus() {
    return Derived::hasMultiplexedBusImpl();
  }

  static int getAddressDigits() {
    // Default address bus width is 20 bits, which is 5 hex digits
    return 5;
  }

  protected:

#ifdef ARDUINO_GIGA
    static constexpr uint32_t PORT_K_DBUS_MASK      = 0xFFFFC03F;
    static constexpr uint32_t PORT_K_DBUS_READ      = 0;
    static constexpr uint32_t PORT_K_DBUS_WRITE     = 0x00001540;

    static constexpr uint32_t PORT_J_DBUS_MASK      = 0x0CFFC000;
    static constexpr uint32_t PORT_J_DBUS_READ      = 0;
    static constexpr uint32_t PORT_J_DBUS_WRITE     = 0x51011555; 
    static constexpr uint32_t PORT_J_DBUS_WRITE_LO  = 0x51010015; // J15, J14, J12, J2, J1, J0
    static constexpr uint32_t PORT_J_DBUS_WRITE_HI  = 0x00011540; // J6, J5, J4, J3

    static constexpr uint32_t PORT_G_DBUS_MASK      = 0xF0FFFFFF;
    static constexpr uint32_t PORT_G_DBUS_READ      = 0;
    static constexpr uint32_t PORT_G_DBUS_WRITE     = 0x05000000;
#endif    

    static constexpr unsigned ClockDivisor = 1;
    static constexpr unsigned ClockHighDelay = 1;
    static constexpr unsigned ClockLowDelay = 1;
    BusDirection busDirection = BusDirection::Input; // Default bus direction is input
    ActiveBusWidth writeBusWidth = ActiveBusWidth::Sixteen; // Default write bus width is 16 bits
    size_t dataBusWidth = 16; // Default data bus width is 16 bits

  static void initPinsCommon() {
    // … shared pin‐setup …
  }

  void setBusDirectionImpl(BusDirection direction, ActiveBusWidth width = ActiveBusWidth::Sixteen) {
    if ((direction == busDirection) && (width == writeBusWidth)) {
      return; // No change needed
    }
    
    if (direction == BusDirection::Input) {
      // Set data bus pins to input
      #if defined(__SAM3X8E__) // Arduino DUE
        PIOB->PIO_ODR = BIT26;      // Pin 22
        PIOA->PIO_ODR = BIT14 | BIT15 | BIT07; // Pins 23, 24, 31
        PIOD->PIO_ODR = BIT00 | BIT01 | BIT02 | BIT03 | BIT06 | BIT09 | BIT10; // Pins 25-29 except 28, plus 30 & 32
        PIOC->PIO_ODR = BIT01 | BIT02 | BIT03 | BIT04 | BIT05; // Pins 33-37
      #elif defined(ARDUINO_GIGA)
        GPIOK->MODER = (GPIOK->MODER & PORT_K_DBUS_MASK) | PORT_K_DBUS_READ;
        GPIOJ->MODER = (GPIOJ->MODER & PORT_J_DBUS_MASK) | PORT_J_DBUS_READ;
        GPIOG->MODER = (GPIOG->MODER & PORT_G_DBUS_MASK) | PORT_G_DBUS_READ;
      #endif
    } else {
      // Set data bus pins to output
      #if defined(__SAM3X8E__)
        if ((width == EightLow) || (width == Sixteen)) {
          // Set data bus pins to OUTPUT
          PIOB->PIO_OER = BIT26;      // Pin 22
          PIOA->PIO_OER = BIT14 | BIT15; // Pins 23, 24
          PIOD->PIO_OER = BIT00 | BIT01 | BIT02 | BIT03 | BIT06; // Pins 25-29 except 28
        }
        if ((width == EightHigh) || (width == Sixteen)) {
          // Set pins to OUTPUT
          PIOD->PIO_OER = BIT09 | BIT10; // Pins 30 & 32
          PIOA->PIO_OER = BIT07; // Pin 31
          PIOC->PIO_OER = BIT01 | BIT02 | BIT03 | BIT04 | BIT05; // Pins 33-37
        }
      #elif defined(ARDUINO_GIGA)
        switch (width) {
          case ActiveBusWidth::EightLow:
            GPIOJ->MODER = (GPIOJ->MODER & PORT_J_DBUS_MASK) | PORT_J_DBUS_WRITE_LO;
            GPIOG->MODER = (GPIOG->MODER & PORT_G_DBUS_MASK) | PORT_G_DBUS_WRITE;
            break;
          case ActiveBusWidth::EightHigh:
            GPIOK->MODER = (GPIOK->MODER & PORT_K_DBUS_MASK) | PORT_K_DBUS_WRITE;
            GPIOJ->MODER = (GPIOJ->MODER & PORT_J_DBUS_MASK) | PORT_J_DBUS_WRITE_HI;
            break;
          case ActiveBusWidth::Sixteen:
            GPIOK->MODER = (GPIOK->MODER & PORT_K_DBUS_MASK) | PORT_K_DBUS_WRITE;
            GPIOJ->MODER = (GPIOJ->MODER & PORT_J_DBUS_MASK) | PORT_J_DBUS_WRITE;
            GPIOG->MODER = (GPIOG->MODER & PORT_G_DBUS_MASK) | PORT_G_DBUS_WRITE;
            break;
          default:
            break;            
        }
      #endif
    }

    busDirection = direction;
    writeBusWidth = width;
  }

  virtual ~HatBase() = default;
};