; -----------------------------------------------------------------------------
; program_regs_286.asm
;
; Registers for 80286 CPUs.
; For 8088, 8086, V20, V30 or 80186 CPUs, use program_regs.asm.
;
; Assembling this file creates a BIN file representing the initial register state.
; Assemble with NASM:
; nasm program_regs.asm -o regs.bin
; -----------------------------------------------------------------------------
%define CPU_286
%include "Arduinox86.inc"
org	0h

; -----------------------------------------------------------------------------
;  Set desired register values here.
; -----------------------------------------------------------------------------
%define CS_REG    0xF000
%define IP_REG    0x0100
%define FLAGS_REG 0x0002

%define MSW       0xFFF0 ; real mode
%define TR_REG    0x0000
%define LDT_REG   0x0000

%define AX_REG    0x0000
%define BX_REG    0x0000
%define CX_REG    0x0000
%define DX_REG    0x0000

%define DS_REG    0x0000
%define SS_REG    0x0000
%define ES_REG    0xC000

%define DI_REG    0x0000
%define SI_REG    0x0000
%define BP_REG    0x0000
%define SP_REG    0xFFF4

%define X0_REG    0x0000
%define X1_REG    0x0100
%define X2_REG    0x002A ; X2 appears to always be 42.
%define X3_REG    0xFFFF
%define X4_REG    0x0000
%define X5_REG    0x0000
%define X6_REG    0x0000
%define X7_REG    0x0000
%define X8_REG    0x0000
%define X9_REG    0x0000

SECTION .data
; -----------------------------------------------------------------------------
;  27 x 16-bit words for register file
;
;  Do not modify - set values above.
; -----------------------------------------------------------------------------
loadall_data:
    dw X0_REG
    dw X1_REG
    dw X2_REG
    dw MSW          ; MSW = 0 → real mode
    dw X3_REG
    dw X4_REG
    dw X5_REG
    dw X6_REG
    dw X7_REG
    dw X8_REG
    dw X9_REG
    dw TR_REG       ; TR
    dw FLAGS_REG    ; FLAGS (bit1=1)
    dw IP_REG       ; IP
    dw LDT_REG      ; LDT
    dw DS_REG       ; DS
    dw SS_REG       ; SS
    dw CS_REG       ; CS
    dw ES_REG       ; ES
    dw DI_REG       ; DI
    dw SI_REG       ; SI
    dw BP_REG       ; BP
    dw SP_REG       ; SP
    dw BX_REG       ; BX
    dw DX_REG       ; DX
    dw CX_REG       ; CX
    dw AX_REG       ; AX

; -----------------------------------------------------------------------------
;  8 x 6-byte descriptors
;
;  Set desired descriptor entries here.
;  By default, the SEG_TO_BASE function is used to set the descriptors to match
;  the segment register values specified above for real mode operation.
; -----------------------------------------------------------------------------

; Access byte
; Bit 0:    Accessed
; Bits 1-3: Type:
;   0=invalid
;   1=available task state segment
;   2=LDT descriptor
;   3=busy task state segment
;   4-7 control descriptor,
;   8-F=invalid
; Bit 4:    S (0=system, 1=code/data)
; Bits 5-6: DPL (0=ring 0, 1=ring 1, 2=ring 2, 3=ring 3)
; Bit 7:    Present

; Default main segment access byte is 0x82.  Not accessed, Type 1, Present.

; RESET defaults: 0x000000, 0xFFFF, 0x82
es_desc:
    DESC_CACHE286 SEG_TO_BASE(ES_REG), 0x0FFFF, 0x82

; RESET defaults: 0xFF0000, 0xFFFF, 0x82
cs_desc:
    DESC_CACHE286 SEG_TO_BASE(CS_REG), 0x0FFFF, 0x82

; RESET defaults: 0x000000, 0xFFFF, 0x82
ss_desc:
    DESC_CACHE286 SEG_TO_BASE(SS_REG), 0x0FFFF, 0x82

; RESET defaults: 0x000000, 0xFFFF, 0x82
ds_desc:
    DESC_CACHE286 SEG_TO_BASE(DS_REG), 0x0FFFF, 0x82

; RESET defaults: 0x000000, 0x0000, 0x00
gdt_desc:
    DESC_CACHE286 0x000000, 0x0FFFF, 0x82

; RESET defaults: 0x000000, 0xFFFF, 0x7F
ldt_desc:
    DESC_CACHE286 0x000000, 0x0FFFF, 0x7F

; RESET defaults: 0x000000, 0xFFFF, 0xFF
idt_desc:
    DESC_CACHE286 0x000000, 0x0FFFF, 0xFF

; RESET defaults: 0x000000, 0x0000, 0xFF
tss_desc:
    DESC_CACHE286 0x000000, 0x0000, 0xFF