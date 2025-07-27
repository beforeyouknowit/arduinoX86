; store_nmi_386_real.asm
; Assemble with nasm: 
; nasm store_nmi_386_real.asm -o store.bin

; Registers are output in turn to IO addresses that can be mapped to offsets into
; the 386 LOADALL structure.

; This routine is intended to run out of an NMI handler that terminates program
; execution. Therefore IP, CS and FLAGS can be popped from the stack.

cpu	386
org	0h

%define IO_BASE   0x80

%macro  IO_OUT 2
    out     byte    IO_BASE + %1, %2
%endmacro

    push    ax
    mov     ax,     08000h    ; set ESE = 1
    out     23h,    al        ; write low byte, unlock
    xchg    al,     ah
    out     22h,    al        ; write low byte of AX
    out     22h,    ax        ; write high byte of AX (now REMAPCFG is writable)

    mov     al,     0FFh      ; Mask off all devices
    out     22h,    al
    pop     ax

    IO_OUT  0x28,   eax       ; EAX
    mov     eax,    cr0       ; CR0
    IO_OUT  0x00,   eax       ; CR0
    mov     eax,    ebx       ; EBX
    IO_OUT  0x1C,   eax       ; EBX
    mov     eax,    ecx       ; ECX
    IO_OUT  0x24,   eax       ; ECX
    mov     eax,    edx       ; EDX
    IO_OUT  0x20,   eax       ; EDX
    mov     eax,    dr6       ; DR6
    IO_OUT  0x2C,   eax       ; DR6
    mov     eax,    dr7       ; DR7
    IO_OUT  0x30,   eax       ; DR7
    pop     ax                ; IP
    IO_OUT  0x08,   ax        ; IP
    pop     ax                ; CS
    IO_OUT  0x4C,   ax        ; CS
    pop     ax                ; EFlags
    IO_OUT  0x04,   ax        ; EFlags
    mov     ax,     ss        ; SS
    IO_OUT  0x48,   ax        ; SS
    mov     eax,    esp       ; ESP
    IO_OUT  0x18,   eax       ; ESP
    mov     ax,     ds        ; DS
    IO_OUT  0x44,   ax        ; DS
    mov     ax,     es        ; ES
    IO_OUT  0x50,   ax        ; ES
    mov     ax,     fs        ; FS
    IO_OUT  0x40,   ax        ; FS
    mov     ax,     gs        ; GS
    IO_OUT  0x3C,   ax        ; GS
    mov     eax,    ebp       ; EBP
    IO_OUT  0x14,   eax       ; EBP
    mov     eax,    esi       ; ESI
    IO_OUT  0x10,   eax       ; ESI
    mov     eax,    edi       ; EDI
    IO_OUT  0x0C,   eax       ; EDI

    mov     ax,     0xFFFF    ; Sent as a signal to end STORE
    out     0xFD,   ax        ; Done!
