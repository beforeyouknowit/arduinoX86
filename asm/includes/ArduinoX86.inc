; ArduinoX86 Copyright 2022-2025 Daniel Balsom
; https://github.com/dbalsom/arduinoX86

; Permission is hereby granted, free of charge, to any person obtaining a
; copy of this software and associated documentation files (the “Software”),
; to deal in the Software without restriction, including without limitation
; the rights to use, copy, modify, merge, publish, distribute, sublicense,
; and/or sell copies of the Software, and to permit persons to whom the
; Software is furnished to do so, subject to the following conditions:

; The above copyright notice and this permission notice shall be included in
; all copies or substantial portions of the Software.

; THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
; IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
; FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
; AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
; LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
; FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
; DEALINGS IN THE SOFTWARE.


; -----------------------------------------------------------------------------
; END macro: All programs should terminate with 'END' which will trigger
;            register readout.
;            On 808X and 8018X, this is a HALT instruction. ArduinoX86 detects
;            the halt state, recovers via NMI and executes the register readout
;            program in the NMI handler.
;            On 80286, this is the STOREALL instruction, which will dump the
;            register state immediately to address 800h.
; -----------------------------------------------------------------------------
%ifdef CPU_8086
    %macro END 0
        hlt             ; HALT instruction for 8086
    %endmacro
%elifdef CPU_80186
    %macro END 0
        hlt             ; HALT instruction for 80186
    %endmacro
%elifdef CPU_286
    %macro END 0
        hlt             ; HALT instruction for 286
    %endmacro
%elifdef CPU_386
    %macro END 0
        hlt             ; HALT instruction for 386
    %endmacro
%else
    %error "END macro: define one of CPU_8086, CPU_80186, or CPU_286 before including"
%endif

; -----------------------------------------------------------------------------
;  6-byte 286 descriptor cache entry
; -----------------------------------------------------------------------------
struc desc_cache286
    .addr_15_00 resw 1 ; Base[15:0]
    .addr_23_16 resb 1 ; Base[23:16]
    .access     resb 1 ; P|DPL|S|Type
    .limit      resw 1 ; Limit[15:0]
endstruc

; Convert a segment value to a base address.
; This is useful for initializing the descriptor cache to match segment
; registers.
%define SEG_TO_BASE(x)  ((x) << 4)

;-------------------------------------------------------------------------------
; DESC_CACHE286 base, limit, access_byte
; Creates a 286 descriptor cache entry for use with LOADALL.
;-------------------------------------------------------------------------------
%macro DESC_CACHE286 3
    istruc desc_cache286
      at desc_cache286.addr_15_00, dw   %1 & 0xFFFF
      at desc_cache286.addr_23_16, db   (%1 >> 16) & 0xFF
      at desc_cache286.access,     db   %3
      at desc_cache286.limit,      dw   %2 & 0xFFFF
    iend
%endmacro

; -----------------------------------------------------------------------------
;  4-byte 386 LOADALL segment structure
; -----------------------------------------------------------------------------
struc segment386
    .seg resw 1 ; Segment value
    .pad resw 1 ; Padding to doubleword bytes
endstruc

%macro SEGMENT386 1
    istruc segment386
      at segment386.seg, dw   %1
      at segment386.pad, dw   0
    iend
%endmacro

; -----------------------------------------------------------------------------
;  6-byte 386 descriptor cache entry
; -----------------------------------------------------------------------------
struc desc_cache386
    .access     resd 1 ; 0|P|DPL|S|Type|A|0|D|0
    .address    resd 1 ; 32-bit address
    .limit      resd 1 ; 32-bit limit
endstruc

;-------------------------------------------------------------------------------
; DESC_CACHE386 access_dword, address, limit
; Creates a 386 descriptor cache entry for use with LOADALL.
;-------------------------------------------------------------------------------
%macro DESC_CACHE386 3
    istruc desc_cache386
      at desc_cache386.access,    dd    %1
      at desc_cache386.address,   dd    %2
      at desc_cache386.limit,     dd    %3
    iend
%endmacro