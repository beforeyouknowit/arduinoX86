
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
#include <BusTypes.h>


// CPU Registers - for new NMI STORE routine
typedef struct __attribute__((packed)) registers1 {
  uint16_t ax;
  uint16_t bx;
  uint16_t cx;
  uint16_t dx;
  uint16_t ip;
  uint16_t cs;
  uint16_t flags;
  uint16_t ss;
  uint16_t sp;
  uint16_t ds;
  uint16_t es;
  uint16_t bp;
  uint16_t si;
  uint16_t di;
} registers1_t;

// CPU Registers - for original STORE routine
typedef struct __attribute__((packed)) registers2 {
  uint16_t ax;
  uint16_t bx;
  uint16_t cx;
  uint16_t dx;
  uint16_t ss;
  uint16_t sp;
  uint16_t flags;
  uint16_t ip;
  uint16_t cs;
  uint16_t ds;
  uint16_t es;
  uint16_t bp;
  uint16_t si;
  uint16_t di;
} registers2_t;


struct __attribute__((packed)) SegmentDescriptor286 {
  uint16_t addr_lo;
  uint8_t addr_hi;
  uint8_t access;
  uint16_t limit;
};

struct __attribute__((packed)) SegmentDescriptor386 {
  uint32_t access;
  uint32_t address;
  uint32_t limit;
};

// CPU Registers - for 286 LOADALL command
struct __attribute__((packed)) Loadall286 {
  uint16_t x0;
  uint16_t x1;
  uint16_t x2;
  uint16_t msw;
  uint16_t x3;
  uint16_t x4;
  uint16_t x5;
  uint16_t x6;
  uint16_t x7;
  uint16_t x8;
  uint16_t x9;
  uint16_t tr;
  uint16_t flags;
  uint16_t ip;
  uint16_t ldt;
  uint16_t ds;
  uint16_t ss;
  uint16_t cs;
  uint16_t es;
  uint16_t di;
  uint16_t si;
  uint16_t bp;
  uint16_t sp;
  uint16_t bx;
  uint16_t dx;
  uint16_t cx;
  uint16_t ax;
  SegmentDescriptor286 es_desc;
  SegmentDescriptor286 cs_desc;
  SegmentDescriptor286 ss_desc;
  SegmentDescriptor286 ds_desc;
  SegmentDescriptor286 gdt_desc;
  SegmentDescriptor286 ldt_desc;
  SegmentDescriptor286 idt_desc;
  SegmentDescriptor286 tss_desc;

  /// @brief Patch the Loadall286 registers from a CallStackFrame.
  void patch_stack_frame(const CallStackFrame& frame) {
    flags = frame.flags;
    cs    = frame.cs;
    ip    = frame.ip;
    sp += 6; // Adjust SP to account for the pushed flags, CS, and IP
  }

  void rewind_ip(uint16_t offset) {
    ip -= offset;
  }
};

#define LOADALL286_ADDRESS 0x800

#define FLAGS_SET_386 0xFFFC0002
#define FLAGS_CLEAR_386 0xFFFF7FD7

struct __attribute__((packed)) Store386 {
  uint32_t eax;
  uint32_t ebx;
  uint32_t ecx;
  uint32_t edx; 
  uint32_t eip;
  uint16_t cs;
  uint16_t cs_pad;
  uint32_t eflags;
  uint16_t ss;
  uint16_t ss_pad;
  uint32_t esp;
  uint16_t ds;
  uint16_t ds_pad;
  uint16_t es;
  uint16_t es_pad;
  uint16_t fs;
  uint16_t fs_pad;
  uint16_t gs;
  uint16_t gs_pad;
  uint32_t ebp;
  uint32_t esi;
  uint32_t edi;
};

struct __attribute__((packed)) Loadall386 {
  uint32_t cr0;
  uint32_t eflags;
  uint32_t eip;
  uint32_t edi;
  uint32_t esi;
  uint32_t ebp;
  uint32_t esp;
  uint32_t ebx;
  uint32_t edx;
  uint32_t ecx;
  uint32_t eax;
  uint32_t dr6;
  uint32_t dr7;
  uint16_t tr;
  uint16_t tr_pad;
  uint16_t ldt;
  uint16_t ldt_pad;
  uint16_t gs;
  uint16_t gs_pad;
  uint16_t fs;
  uint16_t fs_pad;
  uint16_t ds;
  uint16_t ds_pad;
  uint16_t ss;
  uint16_t ss_pad;
  uint16_t cs;
  uint16_t cs_pad;
  uint16_t es;
  uint16_t es_pad;
  SegmentDescriptor386 tss_desc;
  SegmentDescriptor386 idt_desc;
  SegmentDescriptor386 gdt_desc;
  SegmentDescriptor386 ldt_desc;
  SegmentDescriptor386 gs_desc;
  SegmentDescriptor386 fs_desc;
  SegmentDescriptor386 ds_desc;
  SegmentDescriptor386 ss_desc;
  SegmentDescriptor386 cs_desc;
  SegmentDescriptor386 es_desc;

    /// @brief Patch the Loadall386 registers from a CallStackFrame386.
  void patch_stack_frame32(const CallStackFrame32& frame) {
    eflags = frame.eflags;
    cs     = frame.cs;
    eip    = frame.eip;
    esp   += 6; // Adjust ESP to account for the pushed eflags, CS, and EIP
  }

  void rewind_ip(uint32_t offset) {
    eip -= offset;
  }

  // void from_smm(const SmmDump386& smm_dump) {
  //   cr0 = smm_dump.cr0;
  //   eflags = smm_dump.eflags;
  //   eip = smm_dump.eip;
  //   edi = smm_dump.edi;
  //   esi = smm_dump.esi;
  //   ebp = smm_dump.ebp;
  //   esp = smm_dump.esp;
  //   ebx = smm_dump.ebx;
  //   edx = smm_dump.edx;
  //   ecx = smm_dump.ecx;
  //   eax = smm_dump.eax;
  //   dr6 = smm_dump.dr6;
  //   dr7 = smm_dump.dr7;
  //   tr = smm_dump.tr;
  //   ldt = smm_dump.ldt;
  //   gs = smm_dump.gs;
  //   fs = smm_dump.fs;
  //   ds = smm_dump.ds;
  //   ss = smm_dump.ss;
  //   cs = smm_dump.cs;
  //   tss_desc = smm_dump.tss_desc;
  //   idt_desc = smm_dump.idt_desc;
  //   gdt_desc = smm_dump.gdt_desc;
  //   ldt_desc = smm_dump.ldt_desc;
  //   gs_desc = smm_dump.gs_desc;
  //   fs_desc = smm_dump.fs_desc;
  //   ds_desc = smm_dump.ds_desc;
  //   ss_desc = smm_dump.ss_desc;
  //   cs_desc = smm_dump.cs_desc;
  //   es_desc = smm_dump.es_desc;
  // }
};

/// The 386 SMM dump structure. It is similar to LOADALL386, but adds the CR3
/// register. In addition, it writes this structure backwards, like a stack.
struct __attribute__((packed)) SmmDump386 {
  uint32_t cr0;
  uint32_t cr3;
  uint32_t eflags;
  uint32_t eip;
  uint32_t edi;
  uint32_t esi;
  uint32_t ebp;
  uint32_t esp;
  uint32_t ebx;
  uint32_t edx;
  uint32_t ecx;
  uint32_t eax;
  uint32_t dr6;
  uint32_t dr7;
  uint16_t tr;
  uint16_t tr_pad;
  uint16_t ldt;
  uint16_t ldt_pad;
  uint16_t gs;
  uint16_t gs_pad;
  uint16_t fs;
  uint16_t fs_pad;
  uint16_t ds;
  uint16_t ds_pad;
  uint16_t ss;
  uint16_t ss_pad;
  uint16_t cs;
  uint16_t cs_pad;
  uint16_t es;
  uint16_t es_pad;
  SegmentDescriptor386 tss_desc;
  SegmentDescriptor386 idt_desc;
  SegmentDescriptor386 gdt_desc;
  SegmentDescriptor386 ldt_desc;
  SegmentDescriptor386 gs_desc;
  SegmentDescriptor386 fs_desc;
  SegmentDescriptor386 ds_desc;
  SegmentDescriptor386 ss_desc;
  SegmentDescriptor386 cs_desc;
  SegmentDescriptor386 es_desc;

  void normalize_flags() {
    eflags &= FLAGS_CLEAR_386;
    eflags |= FLAGS_SET_386;
  }
};

/// The SMRAM address is fixed on the 386EX from 0x3FE00 to 0x3FFFF.
#define SMRAM_386EX_START_ADDRESS 0x3FE00
#define SMRAM_386EX_DUMP_START 0x3FF14
#define SMRAM_386EX_END_ADDRESS 0x3FFFF
#define SMM_LOAD_CHECKPOINT 0x03FF32

#define SMM_HANDLER_START_ADDRESS 0x038000
#define SMM_HANDLER_END_ADDRESS 0x03FE00

#define LOADALL386_ADDRESS 0x800

