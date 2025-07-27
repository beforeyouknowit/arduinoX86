; -----------------------------------------------------------------------------
; program_regs_386.asm
;
; Registers for 80386 CPUs.
; For 8088, 8086, V20, V30 or 80186 CPUs, use program_regs.asm.
; for 80286 CPUs, use program_regs_286.asm.
;
; Assembling this file creates a BIN file representing the initial register state.
; Assemble with NASM:
; nasm program_regs_386.asm -o regs.bin
; -----------------------------------------------------------------------------
%define CPU_386
%include "Arduinox86.inc"
org	0h

; -----------------------------------------------------------------------------
;  Set desired register values here.
; -----------------------------------------------------------------------------
%define CS_REG      0x1000
%define EIP_REG     0x00000000
%define CR0_REG     0x7FFFFFE0
%define DR6_REG     0x00000000
%define DR7_REG     0x00000000

%define EFLAGS_REG  0x00000002

%define TR_REG      0x0000
%define LDT_REG     0x0000

%define EAX_REG     0x00001234
%define EBX_REG     0x00005678
%define ECX_REG     0x00009ABC
%define EDX_REG     0x0000DEF0

%define SS_REG      0xDEAD
%define DS_REG      0xBEEF
%define ES_REG      0xCAFE
%define FS_REG      0xBABE
%define GS_REG      0xFEEB

%define EDI_REG     0x11111111
%define ESI_REG     0x22222222
%define EBP_REG     0x33333333
%define ESP_REG     0xFFFFFFF4



%define X0_REG      0x00000000
%define X1_REG      0x0000
%define X2_REG      0x002A ; X2 appears to always be 42.
%define X3_REG      0xFFFF
%define X4_REG      0x0000
%define X5_REG      0x0000
%define X6_REG      0x0000
%define X7_REG      0x0000
%define X8_REG      0x0000
%define X9_REG      0x0000

SECTION .data
; -----------------------------------------------------------------------------
;  27 x 16-bit words for register file
;
;  Do not modify - set values above.
; -----------------------------------------------------------------------------
loadall_data:
    dd CR0_REG        ; + 0x00
    dd EFLAGS_REG     ; + 0x04
    dd EIP_REG        ; + 0x08
    dd EDI_REG        ; + 0x0C
    dd ESI_REG        ; + 0x10
    dd EBP_REG        ; + 0x14
    dd ESP_REG        ; + 0x18
    dd EBX_REG        ; + 0x1C
    dd EDX_REG        ; + 0x20
    dd ECX_REG        ; + 0x24
    dd EAX_REG        ; + 0x28
    dd DR6_REG        ; + 0x2C
    dd DR7_REG        ; + 0x30
tr_reg:
    SEGMENT386(TR_REG)  ; + 0x34
    SEGMENT386(LDT_REG) ; + 0x38
    SEGMENT386(GS_REG)  ; + 0x3C
    SEGMENT386(FS_REG)  ; + 0x40
    SEGMENT386(DS_REG)  ; + 0x44
    SEGMENT386(SS_REG)  ; + 0x48
    SEGMENT386(CS_REG)  ; + 0x4C
    SEGMENT386(ES_REG)  ; + 0x50

; -----------------------------------------------------------------------------
;  10 x 12-byte descriptors
;
;  Set desired descriptor entries here.
;  By default, the SEG_TO_BASE function is used to set the descriptors to match
;  the segment register values specified above for real mode operation.
; -----------------------------------------------------------------------------

; Access byte

; Default main segment access word is 0824000h.  Not accessed, Type 1, Present.

tss_desc:
    DESC_CACHE386 00008900h, 0, 0xFFFFFFFF

idt_desc:
    DESC_CACHE386 00000000h, 0, 0xFFFFFFFF

gdt_desc:
    DESC_CACHE386 00000000h, 0, 0xFFFFFFFF

ldt_desc:
    DESC_CACHE386 00008200h, 0, 0xFFFFFFFF

gs_desc:
    DESC_CACHE386 00009300h, SEG_TO_BASE(GS_REG), 0x0000FFFF

fs_desc:
    DESC_CACHE386 00009300h, SEG_TO_BASE(FS_REG), 0x0000FFFF

ds_desc:
    DESC_CACHE386 00009300h, SEG_TO_BASE(DS_REG), 0x0000FFFF

ss_desc:
    DESC_CACHE386 00009300h, SEG_TO_BASE(SS_REG), 0x0000FFFF

cs_desc:
    DESC_CACHE386 0x00009B00, SEG_TO_BASE(CS_REG), 0x0000FFFF
    ; 0x8A4000
es_desc:
    DESC_CACHE386 00009300h, SEG_TO_BASE(ES_REG), 0x0000FFFF









