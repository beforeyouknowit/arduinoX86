; fpu_add_1_plus_2.asm
; 386 + 387 bare-metal: add 1.0 + 2.0 with x87 and put integer result in ECX.
; Assemble: nasm -f bin fpu_add_1_plus_2.asm -o fpu_add.bin

BITS 16
org 100h

start:
    cli

    ; ---- Make DS=ES=SS=CS and set up a small stack ----
    push cs
    pop  ds
    push cs
    pop  es
    push cs
    pop  ss
    mov  sp, stack_top

    ; ---- Initialize CR0 for x87 use on a 386 ----
    ; CR0: MP=bit1, EM=bit2, TS=bit3
    mov  eax, cr0
    and  eax, ~(1 << 2)      ; EM = 0 (allow x87 opcodes)
    or   eax,  (1 << 1)      ; MP = 1
    mov  cr0, eax
    clts                      ; TS = 0 (allow immediate x87 use)

    ; ---- Put FPU into a known state ----
    finit
    fwait                     ; ensure FINIT completed

    ; ---- Compute 1.0 + 2.0 ----
    fld  dword [one_f]        ; ST0 = 1.0
    fld  dword [two_f]        ; ST0 = 2.0, ST1 = 1.0
    faddp st1, st0            ; ST1 = 1.0 + 2.0 = 3.0, pop -> ST0 = 3.0

    ; ---- Convert to integer and store ----
    fistp dword [result]      ; store 3 -> [result], pop
    fwait                     ; make sure store/exception reporting completed

    ; ---- Load into ECX ---
    mov  ecx, [result]

.hang:
    hlt
    jmp  .hang                ; park forever

; ---- Data (aligned) ----
align 4
one_f   dd 0x3f800000         ; 1.0f
two_f   dd 0x40000000         ; 2.0f
result  dd 0

; ---- Tiny stack ----
stack_area:
    times 512 db 0
stack_top: