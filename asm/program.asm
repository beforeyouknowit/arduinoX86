; program.asm
; Compile with nasm to build program.bin for cpu_client
; nasm program.asm -o program.bin
cpu	8086
org	0h

start:
    mov ax, 0x0001
    mov bx, 0x0002
    add ax, bx
    hlt

