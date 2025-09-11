/*
    ArduinoX86 Copyright 2022-2025 Daniel Balsom
    https://github.com/dbalsom/arduinoX86

    Permission is hereby granted, free of charge, to any person obtaining a
    copy of this software and associated documentation files (the "Software"),
    to deal in the Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, sublicense,
    and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER   
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.
*/
#include <Arduino.h>
#include <DebugPrint.h>
#include <cstdarg>

template<typename SerialPort>
void DebugPrintMixin<SerialPort>::debugPrintf(DebugType stage, bool defer, const char* fmt, ...) {
  if (!serial || !filter.isEnabled(stage) || !enabled_) return;

  va_list args;
  va_start(args, fmt);
  int n = vsnprintf(printf_buffer_, BUFFER_SIZE, fmt, args);
  va_end(args);
  if (n <= 0) return;

  // clamp length
  size_t len = size_t(n) < BUFFER_SIZE ? size_t(n) : BUFFER_SIZE - 1;
  printf_buffer_[len] = '\0';

  const char* color = getColor(stage);
  const char* reset = ansi::reset;

  if (defer) {
    int w = snprintf(buffer_ptr_,
                      buffer_remain_,
                      "%s%.*s%s",
                      color,
                      int(len),
                      printf_buffer_,
                      reset);
    if (w > 0) {
      buffer_ptr_        += w;
      buffer_remain_     -= w;
      have_deferred_buffer_ = true;
    }
  } else {
    serial.print(color);
    serial.print(printf_buffer_);
    serial.print(reset);
  }
}

#if defined(ARDUINO_GIGA)
  template class DebugPrintMixin<decltype(Serial2)>;
#else 
  template class DebugPrintMixin<decltype(Serial1)>;
#endif