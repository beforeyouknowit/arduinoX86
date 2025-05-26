; regs.asm
; Compile with nasm to build regs.bin for cpu_client
; nasm regs.asm -o regs.bin

cpu	8086
org	0h

; Specify the initial register state by modifying the values below.
; Assembling this file creates a BIN file representing the initial register state.
; Do not modify the order of the registers or add extra data.
dw 0x1234 ; AX
dw 0x0000 ; BX
dw 0x0000 ; CX
dw 0x0000 ; DX
dw 0x0000 ; SS
dw 0xFFFE ; SP
dw 0xF002 ; FLAGS
dw 0x0100 ; IP
dw 0xF000 ; CS
dw 0x0000 ; DS
dw 0x0000 ; ES
dw 0x0000 ; BP
dw 0x0000 ; SI
dw 0x0000 ; DI
