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

// This module defines debug type filters so we can enable or disable specific
// types of debug messages.

enum class DebugType : uint32_t {
    STATE      = 1u << 0,  // DEBUG_STATE
    RESET      = 1u << 1,  // DEBUG_RESET
    SETUP      = 1u << 2,  // DEBUG_SETUP
    VECTOR     = 1u << 3,  // DEBUG_VECTOR
    ID         = 1u << 4,  // DEBUG_ID
    LOAD       = 1u << 5,  // DEBUG_LOAD
    LOAD_DONE  = 1u << 6,  // DEBUG_LOAD_DONE
    EXECUTE    = 1u << 7,  // DEBUG_EXECUTE
    STORE      = 1u << 8,  // DEBUG_STORE
    FINALIZE   = 1u << 9,  // DEBUG_FINALIZE
    INSTR      = 1u << 10, // DEBUG_INSTR
    EMU        = 1u << 11, // DEBUG_EMU
    QUEUE      = 1u << 12, // DEBUG_QUEUE
    TSTATE     = 1u << 13, // DEBUG_TSTATE
    PIN_CMD    = 1u << 14, // DEBUG_PIN_CMD
    BUS        = 1u << 15, // DEBUG_BUS
    PROTO      = 1u << 16, // DEBUG_PROTO (0 in macro, but reserving bit)
    CMD        = 1u << 17  // DEBUG_CMD (0 in macro, but reserving bit)
};

class DebugFilter {
  uint32_t enabledTypes = 0u; // All disabled by default

public:
  void setStageEnabled(DebugType debug_type, bool enabled) {
    if (enabled)
      enabledTypes |= static_cast<uint32_t>(debug_type);
    else
      enabledTypes &= ~static_cast<uint32_t>(debug_type);
  }

  bool isEnabled(DebugType debug_type) const {
    return (enabledTypes & static_cast<uint32_t>(debug_type)) != 0;
  }
};