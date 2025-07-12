#pragma once
#include <bus_emulator/BusEmulator.h>

// Null backend: does nothing and returns zero
class NullBackend : public IBusBackend {
public:
  uint8_t  read_u8(uint32_t) override { return 0; }
  uint16_t read_u16(uint32_t) override { return 0; }
  uint16_t read_bus(uint32_t, bool) override { return 0; }
  void     write_u8(uint32_t, uint8_t) override {}
  void     write_u16(uint32_t, uint16_t) override {}
  void     write_bus(uint32_t, uint16_t, bool) override {}
  uint8_t  io_read_u8(uint16_t) override { return 0; }
  uint16_t io_read_u16(uint16_t) override { return 0; }
  void     io_write_u8(uint16_t, uint8_t) override {}
  void     io_write_u16(uint16_t, uint16_t) override {}
  void     set_memory(uint32_t, const uint8_t*, size_t) override {}
  void     randomize_memory() override {}
  void     debug_mem(uint32_t, size_t) override {}
};
