; program.asm
; Compile with nasm to build program.bin for cpu_client
; nasm program.asm -o program.bin
cpu	386
org	0h

start:
    jmp -100h

    MOV AX, 08000H ; Enable expanded I/O space
    OUT 23H, AL ; and unlock the re-map bits
    XCHG AL, AH
    OUT 22H, AL
    OUT 22H, AX
    mov dx, 0F438h
    mov ax, 0380h
    out dx, ax
    mov dx, 0F43Eh ;
    mov ax, 0
    out dx, ax
    mov ax, 1
    mov dx, 0F43Ch
    out dx, ax
