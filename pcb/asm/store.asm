; store.asm
; Original routine by Andreas Jonsson
; https://github.com/andreas-jonsson/virtualxt/tree/develop/tools/validator/pi8088
;
; Assemble with nasm: 
; nasm store.asm -o store.bin

; Registers are output in turn to dummy IO addresses, intercepted by the validator 
; program. End of the routine is indicated by a write to IO address 0xFD.

; Since there is no direct 'MOV rm, flags' or 'MOV rm, ip' instruction, we push 
; these  two registers to the stack and intercept memory writes to the dummy stack
; space defined at 00000-00003.

cpu	8086
org	0h

    out     0xFE, ax        ; AX
    mov     ax, bx
    out     0xFE, ax        ; BX
    mov     ax, cx
    out     0xFE, ax        ; CX
    mov     ax, dx
    out     0xFE, ax        ; DX

    mov     ax, ss
    out     0xFE, ax        ; SS
    mov     ax, sp
    out     0xFE, ax        ; SP

    mov     ax, 0
    mov     ss, ax
    mov     ax, 4
    mov     sp, ax          ; Set up 4 bytes of stack for flags and IP.
    pushf                   ; Capture flags
    call    _ip             ; We capture IP when it is pushed to the stack on CALL.
                            ; We then adjust it by 24 bytes to the start of the store procedure.
_ip:
    mov     ax, cs
    out     0xFE, ax        ; CS
    mov     ax, ds
    out     0xFE, ax        ; DS
    mov     ax, es
    out     0xFE, ax        ; ES
    mov     ax, bp
    out     0xFE, ax        ; BP
    mov     ax, si
    out     0xFE, ax        ; SI
    mov     ax, di
    out     0xFE, ax        ; DI

    mov     al, 0xFF        ; Sent as a signal to the validator program that we are done.
    out     0xFD, al        ; Done!
