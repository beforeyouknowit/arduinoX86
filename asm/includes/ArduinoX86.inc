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
        hlt             ; HALT instruction for 8086
    %endmacro
%elifdef CPU_286
    %macro END 0
        db 0xF1, 0x0F, 0x05   ; STOREALL for 80286
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