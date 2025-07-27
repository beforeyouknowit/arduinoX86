; program.asm
; Compile with nasm to build program.bin for cpu_client
; nasm program.asm -o program.bin
cpu	386
org	0h

start:
    mov   eax, dr7
    or    eax, 000004000h
    mov   dr7, eax
    nop
    nop
    nop
    nop
    nop
    nop
    nop
    nop
    nop
    nop

    hlt


