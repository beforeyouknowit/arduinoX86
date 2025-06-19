#pragma once
#include <Arduino.h>

class Display {
public:
  virtual void init() {}
  virtual int rows() const { return 0; }
  virtual void updateCell(int line, int col, uint16_t color, const char* text) {}
  virtual uint16_t makeColor(uint8_t r, uint8_t g, uint8_t b) {
    return ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3);
  }
  virtual void trackFrame() {}
  virtual void trackUpdate() {}
  virtual void flush() {}
  virtual ~Display() = default;
};