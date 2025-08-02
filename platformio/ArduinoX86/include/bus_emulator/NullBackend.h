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
#include <bus_emulator/IBusBackend.h>
#include <cstddef>

// Null backend: does nothing and returns zero
class NullBackend : public IBusBackend {
public:
  IBusBackendType type() const override {
    return IBusBackendType::Null;
  }

  size_t   size() const override { return 0; }
  uint8_t  read_u8(uint32_t) override { return 0; }
  uint16_t read_u16(uint32_t) override { return 0; }
  uint16_t read_bus(uint32_t, bool) override { return 0; }
  uint8_t *get_ptr(uint32_t) override { return NULL; }
  void     write_u8(uint32_t, uint8_t) override {}
  void     write_u16(uint32_t, uint16_t) override {}
  void     write_bus(uint32_t, uint16_t, bool) override {}
  uint8_t  io_read_u8(uint16_t) override { return 0; }
  uint16_t io_read_u16(uint16_t) override { return 0; }
  void     io_write_u8(uint16_t, uint8_t) override {}
  void     io_write_u16(uint16_t, uint16_t) override {}
  void     set_memory(uint32_t, const uint8_t*, size_t) override {}
  void     set_strategy(DefaultStrategy, uint32_t, uint32_t) override {}
  void     randomize_memory(uint32_t) override {}
  void     debug_mem(uint32_t, size_t) override {}
};
