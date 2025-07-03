; -----------------------------------------------------------------------------
; program_regs.asm
;
; Registers for 8088, 8086, V20, V30 and 80186 CPUs.
; For 80286 CPUs, use program_regs_286.asm.
;
; Assembling this file creates a BIN file representing the initial register state.
; Assemble with NASM:
; nasm program_regs.asm -o regs.bin
; -----------------------------------------------------------------------------
%define CPU_8086
%include "ArduinoX86.inc"
org	0h

; Specify the initial register state here by modifying these defines.
; -----------------------------------------------------------------------------
%define CS_REG    0xF000
%define IP_REG    0x0100
%define FLAGS_REG 0xF002

%define AX_REG    0x0000
%define BX_REG    0x0000
%define CX_REG    0x0000
%define DX_REG    0x0000

%define DS_REG    0x0000
%define SS_REG    0x0000
%define ES_REG    0xC000

%define SI_REG    0x0000
%define DI_REG    0x0000

%define BP_REG    0x0000
%define SP_REG    0xFFFE

; Do not modify the order of the registers or add extra data.
; -----------------------------------------------------------------------------
SECTION .data
  dw AX_REG ; AX
  dw BX_REG ; BX
  dw CX_REG ; CX
  dw DX_REG ; DX
  dw IP_REG ; IP
  dw CS_REG ; CS
  dw FLAGS_REG ; FLAGS
  dw SS_REG ; SS
  dw SP_REG ; SP
  dw DS_REG ; DS
  dw ES_REG ; ES
  dw BP_REG ; BP
  dw SI_REG ; SI
  dw DI_REG ; DI
