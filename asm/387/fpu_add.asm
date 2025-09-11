; fpu_add.asm
; Assemble with: nasm -f bin fpu_add.asm -o fpu_add.bin

BITS 16
org 0x100             ; .COM-style origin if running under DOS

start:
    finit             ; initialize FPU

    fld1              ; push constant 1.0 onto FPU stack
    fld1              ; push constant 1.0 again
    fadd st0, st0     ; st0 = 1.0 + 1.0 = 2.0
    fadd              ; st1 = 1.0 + st0(2.0) -> st1=3.0, st0 popped

    fistp dword [result]   ; store integer 3 into memory

    mov ecx, [result]      ; load result into ECX

    hlt               ; stop execution

result: dd 0