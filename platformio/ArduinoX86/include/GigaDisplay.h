/*
    Arduino8088 Copyright 2022-2025 Daniel Balsom
    https://github.com/dbalsom/arduino_8088

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


#include <Arduino.h>
#include "Arduino_GigaDisplay_GFX.h"
#include "Display.h"
#include "Vga.h"

class GigaDisplay : public Display {

private:
  static constexpr uint8_t textSize = 4;
  static constexpr uint8_t verticalPadding = 4;
  static constexpr uint8_t cols = 2;
  static constexpr uint8_t topMargin = 20;
  static constexpr unsigned long frameIntervalMs = 16;

  static constexpr uint16_t white = 0xFFFF; // White color in RGB565 format
  static constexpr uint16_t black = 0x0000; // Black color in RGB565 format
  static constexpr uint16_t gray75 = 0xC618; // 75% gray in RGB565 format
  static constexpr uint16_t gray50 = 0x8410; // 50% gray in RGB565 format
  static constexpr uint16_t gray33 = 0x630c; // 33% gray in RGB565 format
  static constexpr uint16_t gray25 = 0x4208; // 25% gray in RGB565 format

  GigaDisplay_GFX &display;
  Vga *vga_;
  bool _isInitialized = false;
  int _rows;

  unsigned long lastFpsTime = 0;
  int framesDrawn = 0;
  float fps = 0.0;

  unsigned long lastUpdateTime = 0;
  int updatesCount = 0;
  float updatesPerSecond = 0.0;

  void drawHLine2x(int x, int y, int w, uint16_t color) {
    display.drawFastHLine(x * 2, y * 2, w * 2, color);
    display.drawFastHLine(x * 2, y * 2 + 1, w * 2, color);
  }

  void drawVLine2x(int x, int y, int h, uint16_t color) {
    display.drawFastVLine(x * 2, y * 2, h * 2, color);
    display.drawFastVLine(x * 2 + 1, y * 2, h * 2, color);
  }

public:
  GigaDisplay(GigaDisplay_GFX& ref) : display(ref) {
    //vga_ = new Vga();
  }

  void init() override {
    display.begin();
    display.setRotation(1);
    display.fillScreen(gray50);
    display.setTextWrap(false);

    int cellHeight = textSize * 8 + verticalPadding;
    _rows = (display.height() - topMargin) / cellHeight;

    display.flush();
    _isInitialized = true;
    //updateCell(0, 0, display.color565(255, 255, 255), "Initialized");
    drawFrame(8, 8, 384, 224, white, gray33);
  }

  int rows() const override {
    return _rows;
  }

  void drawFrame(int x, int y, int width, int height, uint16_t color, uint16_t fillColor) {
    if (!_isInitialized) { return; }

    // Draw top line.
    drawHLine2x(x + 1, y, width - 2, color);
    
    // Draw bottom line.
    drawHLine2x(x + 1, y + (height - 1), width - 2, color);
    
    // Draw left line.
    drawVLine2x(x, y + 1, height - 2, color);

    // Draw right line.
    drawVLine2x(x + (width - 1), y + 1, height - 2, color);

    display.fillRect((x * 2) + 2, (y * 2) + 2, (width * 2) - 4, (height * 2) - 4, fillColor);
  }

  void updateCell(int line, int col, uint16_t color, const char* text) override {
    return;
    if (!_isInitialized) { return; }
    //DEBUG_SERIAL.println("in updateCell()...");
    int cellHeight = textSize * 8 + verticalPadding;
    int cellWidth = display.width() / cols;

    int x = col * cellWidth;
    int y = topMargin + line * cellHeight;

    display.fillRect(x, y, cellWidth, cellHeight, display.color565(0, 0, 0));
    int textWidth = 6 * strlen(text) * textSize;
    int cursorX = x + (cellWidth - textWidth) / 2;
    int cursorY = y + (cellHeight - textSize * 8) / 2;

    // DEBUG_SERIAL.println("print() with text: " + String(text) + 
    //                     ", cursorX: " + String(cursorX) + 
    //                     ", cursorY: " + String(cursorY));
    display.setCursor(cursorX, cursorY);
    display.setTextSize(textSize);
    display.setTextColor(color);
    display.print(text);
    //display.flush();
  }

  uint16_t makeColor(uint8_t r, uint8_t g, uint8_t b) override {
    if (!_isInitialized) { return 0; }
    return display.color565(r, g, b);
  }

  void trackFrame() override {
    framesDrawn++;
    unsigned long now = millis();
    if (now - lastFpsTime >= 1000) {
      fps = framesDrawn / ((now - lastFpsTime) / 1000.0);
      lastFpsTime = now;
      framesDrawn = 0;
    }
  }

  void trackUpdate() override {
    updatesCount++;
    unsigned long now = millis();
    if (now - lastUpdateTime >= 1000) {
      updatesPerSecond = updatesCount / ((now - lastUpdateTime) / 1000.0);
      lastUpdateTime = now;
      updatesCount = 0;
    }
  }

  void flush() override {
    if (!_isInitialized) { return;}
    display.flush();
  }

  void drawVgaFrame(uint8_t *frame_buffer) {
    display.drawBitmap(0, 0, frame_buffer, display.width(), display.height(), white);
  }

  Vga *vga() {
    return vga_;
  }
};