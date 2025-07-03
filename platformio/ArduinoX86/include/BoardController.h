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
#include <hats/Pins.h>

template<typename Board, typename Hat>
class BoardController {
  Board& board;
  Hat hat;  // Add Hat instance to maintain state

public:
  explicit BoardController(Board& b) : board(b), hat() {}
  
  // Constructor that allows passing Hat constructor parameters
  template<typename... HatArgs>
  explicit BoardController(Board& b, HatArgs&&... hatArgs) : board(b), hat(std::forward<HatArgs>(hatArgs)...) {}CpuResetResult resetCpu() {
    return Hat::template resetCpu<Board>(board);
  }

  Board& getBoard() {
    return board;
  }

  void tickCpu() {
    Hat::tickCpu();
  }

  static int getAddressBusWidth() {
    return Hat::getAddressBusWidth();
  }

  static int getAddressDigits() {
    return Hat::getAddressDigits();
  }

  static bool hasSegmentStatus() {
    return Hat::hasSegmentStatus();
  }

  static BusStatus decodeBusStatus(uint8_t status_byte) {
    return Hat::decodeBusStatus(status_byte);
  }

  static TCycle getNextCycle(TCycle current_cycle, BusStatus current_status, BusStatus latched_status) {
    return Hat::getNextCycle(current_cycle, current_status, latched_status);
  }

  uint16_t readDataBus(ActiveBusWidth width, bool peek = false) {
    return hat.readDataBus(width, peek);
  }

  void writeDataBus(uint16_t data, ActiveBusWidth width) {
    hat.writeDataBus(data, width);
  }

  uint32_t readAddressBus(bool peek) {
    return hat.readAddressBus(peek);
  }

  static void writePin(OutputPin pin, bool value) {
    Hat::writePin(pin, value);
  }

  uint8_t readCpuStatusLines() {
    return hat.readCpuStatusLines();
  }

  uint8_t readBusControllerCommandLines() {
    return hat.readBusControllerCommandLines();
  }

  uint8_t readBusControllerControlLines() {
    return hat.readBusControllerControlLines();
  }

  bool cpuIsReading(BusTransferType &read_type) const {
    return Hat::cpuIsReading(read_type);
  }

  bool cpuIsWriting(BusTransferType &write_type) const {
    return Hat::cpuIsWriting(write_type);
  }

  static const char* getBusStatusString(BusStatus status) {
    return Hat::getBusStatusString(status);
  }

  static const char* getBusStatusColor(BusStatus status) {
    return Hat::getBusStatusColor(status);
  }

  static const char* getTCycleString(TCycle cycle) {
    return Hat::getTCycleString(cycle);
  }

  static bool hasMultiplexedBus() {
    return Hat::hasMultiplexedBus();
  }

  static bool readBHEPin() {
    return Hat::readBHEPin();
  }

  bool readALEPin() {
    return hat.readALEPin();
  }

  bool readMRDCPin() {
    return hat.readMRDCPin();
  }

  bool readAMWCPin() {
    return hat.readAMWCPin();
  }

  bool readMWTCPin() {
    return hat.readMWTCPin();
  }

  bool readIORCPin() {
    return hat.readIORCPin();
  }

  bool readIOWCPin() {
    return hat.readIOWCPin();
  }

  bool readAIOWCPin() {
    return hat.readAIOWCPin();
  }
};