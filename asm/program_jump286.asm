; program.asm
; Compile with nasm to build program.bin for cpu_client
; nasm program.asm -o program.bin
cpu	286
org	0h

start:
    jmp myjump
myjump:
    mov ax, 0x1
    mov bx, 0x2
    add ax, bx
    hlt


