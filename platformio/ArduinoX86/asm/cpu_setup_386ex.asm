; cpu_setup_386ex.asm
cpu	386
org	0h

start:
    jmp   -100h
    times 8 nop
    mov   ax, 08000H    ; Enable expanded I/O space
    out   23H, al       ; unlock the re-map bits
    xchg  al, ah
    out   22H, al       ; Knock on the port
    out   22H, ax       ; Knock again with 16-bits to set
    mov   dx, 0F438h    ; UCSADL - Chip-select Low Address
    mov   ax, 0380h
    out   dx, ax
    mov   dx, 0F43Eh    ; UCSMSKH - Chip-select High Mask
    mov   ax, 0
    out   dx, ax
    mov   ax, 1
    mov   dx, 0F43Ch    ; UCSMSKL - Chip-select Low Mask
    out   dx, ax
