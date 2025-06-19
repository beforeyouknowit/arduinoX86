; program.asm
; Compile with nasm to build program.bin for cpu_client
; nasm program.asm -o program.bin
cpu	8086
org	0h

start:

    fldcw [fpu_cw]
    fwait

    ; Load num1 onto FPU stack
    fld qword [num1]          ; ST(0) = num1
    fwait

    ; Load num2 onto FPU stack
    fld qword [num2]          ; ST(0) = num2, ST(1) = num1
    fwait

    ; Add ST(0) and ST(1), result in ST(0), pop stack
    faddp st1, st0            ; ST(1) = ST(1) + ST(0), pop -> ST(0) now contains sum
    fwait

    ; Store result from ST(0) to memory
    fstp qword [result]       ; store and pop
    fwait

    hlt

; --- data section below code ---
    align 2

num1    dq 1.0
num2    dq 2.0
result  dq 0.0
fpu_cw  dw 0x0000