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

#include <CpuTypes.h>
#include <BusTypes.h>
#include <shields/Pins.h>

template<typename Board, typename Shield>
class BoardController {
  Board& board;
  Shield shield;  // Add Shield instance to maintain state

public:
  explicit BoardController(Board& b) : board(b), shield() {}
  
  // Constructor that allows passing Shield constructor parameters
  template<typename... HatArgs>
  explicit BoardController(Board& b, HatArgs&&... hatArgs) : board(b), shield(std::forward<HatArgs>(hatArgs)...) {}
  
  CpuResetResult resetCpu() {
    return shield.resetCpu(board);
  }

  Board& getBoard() {
    return board;
  }

  void tickCpu() {
    shield.tickCpu();
  }

  static int getAddressBusWidth() {
    return Shield::getAddressBusWidth();
  }

  static int getAddressDigits() {
    return Shield::getAddressDigits();
  }

  static bool hasSegmentStatus() {
    return Shield::hasSegmentStatus();
  }

  static BusStatus decodeBusStatus(uint8_t status_byte) {
    return Shield::decodeBusStatus(status_byte);
  }

  static TCycle getNextCycle(TCycle current_cycle, BusStatus current_status, BusStatus latched_status) {
    return Shield::getNextCycle(current_cycle, current_status, latched_status);
  }

  uint16_t readDataBus(ActiveBusWidth width, bool peek = false) {
    return shield.readDataBus(width, peek);
  }

  void writeDataBus(uint16_t data, ActiveBusWidth width) {
    shield.writeDataBus(data, width);
  }

  uint32_t readAddressBus(bool peek) {
    return shield.readAddressBus(peek);
  }

  static void writePin(OutputPin pin, bool value) {
    Shield::writePin(pin, value);
  }

  uint8_t readCpuStatusLines() {
    return shield.readCpuStatusLines();
  }

  uint8_t readBusControllerCommandLines() {
    return shield.readBusControllerCommandLines();
  }

  uint8_t readBusControllerControlLines() {
    return shield.readBusControllerControlLines();
  }

  bool cpuIsReading(BusTransferType &read_type) const {
    return Shield::cpuIsReading(read_type);
  }

  bool cpuIsWriting(BusTransferType &write_type) const {
    return Shield::cpuIsWriting(write_type);
  }

  static const char* getBusStatusString(BusStatus status) {
    return Shield::getBusStatusString(status);
  }

  static const char* getBusStatusColor(BusStatus status) {
    return Shield::getBusStatusColor(status);
  }

  static const char* getTCycleString(TCycle cycle) {
    return Shield::getTCycleString(cycle);
  }

  static bool hasMultiplexedBus() {
    return Shield::hasMultiplexedBus();
  }

  bool readBHEPin() {
    return shield.readBHEPin();
  }

  bool readALEPin() {
    return shield.readALEPin();
  }

  bool readLockPin() {
    return shield.readLockPin();
  }

  bool readReadyPin() {
    return shield.readReadyPin();
  }

  bool readMRDCPin() {
    return shield.readMRDCPin();
  }

  bool readAMWCPin() {
    return shield.readAMWCPin();
  }

  bool readMWTCPin() {
    return shield.readMWTCPin();
  }

  bool readIORCPin() {
    return shield.readIORCPin();
  }

  bool readIOWCPin() {
    return shield.readIOWCPin();
  }

  bool readAIOWCPin() {
    return shield.readAIOWCPin();
  }
};