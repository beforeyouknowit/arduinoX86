; colorful 128‑byte Mandelbrot fractal in NASM syntax
; assembled as a .COM:   nasm -f bin mandelbrot.asm -o mandelbrot.com

bits 16
org 0x100

;--------------------------------------------------------------------
; build our constants
%define fBits        8
%assign fMax        1
%rep fBits
  %assign fMax      fMax * 2
%endrep                    ; now fMax = 2^fBits = 256

%define MaxIterations 16

;--------------------------------------------------------------------
; GetColor macro: compute escape “color” value in AX
%macro GetColor 0
    mov    cl,32
    xor    si,si

.ColorLoop:
    sub    si,ax
    add    si,[Cr]
    mov    ax,[Zr]
    mov    bp,[Zi]
    imul   bp
    shrd   ax,dx,fBits-1
    add    ax,[Ci]
    mov    [Zi],ax
    mov    [Zr],si

    inc    cl
    cmp    cl,MaxIterations+32
    jz     short .GotColor

    imul   ax
    shrd   ax,dx,fBits
    mov    bp,ax
    xchg   si,ax
    imul   ax
    shrd   ax,dx,fBits
    add    bp,ax
    xchg   si,ax
    cmp    bp,4*fMax
    jl     short .ColorLoop

.GotColor:
%endmacro

;--------------------------------------------------------------------
; Mandelbrot macro: iterate over screen pixels
%macro Mandelbrot 0
    mov    ah, -2*fMax/256
    mov    [Cr],ax
    mov    [Ci],ax

    mov    cx,200        ; 200 lines (height)
.VertLoop:
    mov    bx,320        ; 320 columns (width)
    push   cx

.HorizLoop:
    mov    [Zr],ax
    mov    [Zi],ax
    GetColor
    mov    ax,cx
    stosb                ; write AL to [ES:DI], increments DI
    mov    al, 4*fMax/320
    add    [Cr],ax
    dec    bx
    jnz    short .HorizLoop

    mov    al, fMax*4/200
    add    [Ci],al
    mov    ax, -2*fMax
    mov    [Cr],ax

    pop    cx
    loop   .VertLoop
%endmacro

;--------------------------------------------------------------------
; program entry
Start:
    ; switch to 320×200×256‑color mode
    ;mov    al,0x13
    ;int    0x10

    ; point ES at VGA frame buffer
    mov    ax,0xA000
    mov    es,ax
    xor    di,di

    ; draw it
    Mandelbrot

    ; wait for key
    ;xor    ah,ah
    ;int    0x16

    ; back to 80×25 text mode
    ;mov    ax,0x0003
    ;int    0x10

    hlt

;--------------------------------------------------------------------
; storage for our four 16‑bit variables
Cr:  dw 0
Ci:  dw 0
Zr:  dw 0
Zi:  dw 0