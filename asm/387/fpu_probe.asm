; fpu_sw_status_to_dx.asm
; Bare-metal x87 presence check and copy FPU status word into DX.
; Result: ECX = 1 if FPU present (and DX = status), else ECX = 0 (DX = 0xFFFF)

BITS 16
org 100h

start:
    cli

    ; Make DS=ES=SS=CS so labels are addressable safely
    push    cs
    pop     ds
    push    cs
    pop     es
    push    cs
    pop     ss

    mov     sp, stack_top

    ; Defaults
    xor     eax, eax              ; EAX = 0 (assume no FPU)
    mov     dx, 0FFFFh            ; DX = sentinel if no FPU

    ; ---- Presence test: memory form (doesn't touch AX) ----
    mov     word [fpu_sw], 0x5A5A
    fnstsw  [fpu_sw]      ; if no x87, 0x5A5A remains

    cmp     word [fpu_sw], 0x5A5A
    je      .no_fpu

    ; Optional confirm: init and verify a second store happens
    fninit
    mov     word [fpu_sw], 0xFFFF
    fnstsw  [fpu_sw]
    cmp     word [fpu_sw], 0xFFFF
    je      .no_fpu            ; didn't overwrite -> treat as no FPU

    ; ---- Copy status into DX and report success ----
    fnstcw  [fpu_cw]            ; store control word
    mov     cx, [fpu_cw]        ; load into CX
    mov     dx, [fpu_sw]        ; load status word into DX
    mov     eax, 1              ; EAX = 1 (FPU present)

.no_fpu:
.hang:
    hlt
    jmp  .hang

; ---- Data ----
align 2
fpu_sw  dw 0
fpu_cw  dw 0

; ---- Stack ----
stack_area:
    times 512 db 0
stack_top: