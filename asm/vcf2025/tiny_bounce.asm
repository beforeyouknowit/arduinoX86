; Tiny 64x64 Bounce for ArduinoX86
; by Plex

%define DOS 0

org 100h

%if DOS
    mov     al, 13h
    int     10h
%endif
    push    cs
    pop     ds

    push    0xA000
    pop     es

    mov     si, 255
    mov     bp, 139

    mov     ax, 1
    mov     cx, 0
    mov     dx, 0
    mov     bx, 64 * 255
    mov     di, 64 * 139

    mov     word [0], -255
    mov     word [2], -139

start:
    add     cx, si
    test    ch, 0xC0
    jz      skip1
    sub     cx, si
    neg     si
skip1:

    add     dx, bp
    test    dh, 0xC0
    jz      skip2
    sub     dx, bp
    neg     bp
skip2:

    pusha
    movzx   di, ch
    shl     di, 6
    shr     dx, 8
    add     di, dx

    add     byte [es:di], al

    popa

    xchg    cx, bx
    xchg    dx, di
    xchg    [0], si
    xchg    [2], bp
    neg     ax

    jmp     start