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

#define OPCODE_NOP 0x90
#define OPCODE_80NOP 0x00
#define OPCODE_DOUBLENOP 0x9090
#define OPCODE_DOUBLE_80NOP 0x0000

#define MODRM_OP(M) (((M & 0b00111000) >> 3) & 0x07)

#define GRP1 105
#define GRP2A 106
#define GRP2B 110
#define GRP3 107
#define GRP4 108
#define GRP5 109
#define IS_GRP_OP(O) ((OPCODE_REFS[O] >= GRP1) && (OPCODE_REFS[O] <= GRP2B))

extern const char * const OPCODE_STRS[];
extern const char * const OPCODE_STRS_GRP1[];
extern const char * const OPCODE_STRS_GRP2A[];  
extern const char * const OPCODE_STRS_GRP2B[];
extern const char * const OPCODE_STRS_GRP3[];
extern const char * const OPCODE_STRS_GRP4[];
extern const char * const OPCODE_STRS_GRP5[];
extern const char * const OPCODE_8080_STRS[];
extern const uint8_t OPCODE_REFS[];
extern const uint8_t OPCODE_8080_REFS[];
