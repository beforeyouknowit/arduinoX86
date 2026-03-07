; cpu_setup_386ex.asm
cpu	386
org	0h

CFG_ENABLE_EXTENDED EQU (1 << 15)
CFG_REMAP_SERIAL1   EQU (1 << 6)
CFG_REMAP_SERIAL0   EQU (1 << 5)
CFG_REMAP_PIC2      EQU (1 << 4)
CFG_REMAP_PIC1      EQU (1 << 3)
CFG_REMAP_DMA2      EQU (1 << 2)
CFG_REMAP_TIMER     EQU (1 << 0)

CFG_REMAP_ALL       EQU (CFG_REMAP_SERIAL1 | CFG_REMAP_SERIAL0 | CFG_REMAP_PIC2 | CFG_REMAP_PIC1 | CFG_REMAP_DMA2 | CFG_REMAP_TIMER)

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

    mov   al, CFG_REMAP_ALL
    out   22H, al       ; Remap all peripherals into extended IO space...
    mov   al, 0
    out   23H, al       ; ... and then disable the extended IO space

    mov   dx, 0F820H    ; P1CFG
    mov   al, 0FFH      ; Connect all pins.
    out   dx, al
