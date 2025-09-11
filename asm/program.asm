; program.asm
; Compile with nasm to build program.bin for cpu_client
; nasm program.asm -o program.bin

        cpu 386
        bits 16
        org 100h

start:
        mov ax, 01234h
        hlt
