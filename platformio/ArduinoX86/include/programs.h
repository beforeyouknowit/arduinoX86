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

#include <cstdint>
#include <InlineProgram.h>

extern InlineProgram SETUP_PROGRAM_86;
extern InlineProgram SETUP_PROGRAM_186;
extern InlineProgram SETUP_PROGRAM_386EX;
extern InlineProgram LOAD_PROGRAM;
extern InlineProgram LOAD_PROGRAM_286;
extern InlineProgram LOAD_PROGRAM_386;
extern InlineProgram LOAD_PROGRAM_SMM_386;
extern InlineProgram CPUID_PROGRAM;
extern InlineProgram EMU_ENTER_PROGRAM;
extern InlineProgram EMU_EXIT_PROGRAM;
extern InlineProgram JUMP_VECTOR;
extern InlineProgram NMI_VECTOR;
extern InlineProgram STOREALL_PROGRAM;
extern InlineProgram STOREALL_PROGRAM_386;
extern InlineProgram STORE_PROGRAM_NMI;
extern InlineProgram STORE_PROGRAM_NMI_386;
extern InlineProgram STORE_PROGRAM_INLINE;
extern InlineProgram NEC_PREFETCH_PROGRAM;

#define NMI_VECTOR_SIZE 4