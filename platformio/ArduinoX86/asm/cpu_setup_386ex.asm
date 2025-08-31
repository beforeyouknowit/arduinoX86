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
    mov   ax, 0380h     ; 0000_0011_1000_0000: Address 0, SMM bit clear, Bus Size 16, Memory, Ready Enabled, 0 wait states
    out   dx, ax
    mov   dx, 0F43Eh    ; UCSMSKH - Chip-select High Mask
    mov   ax, 0
    out   dx, ax
    mov   ax, 1         ; Enable the chip select channel
    mov   dx, 0F43Ch    ; UCSMSKL - Chip-select Low Mask
    out   dx, ax
