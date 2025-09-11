; plasma.asm — integer sine-LUT plasma, fixed (no tiles, smooth time)
; 16‑bit .COM, NASM syntax

        org     100h

WIDTH   equ 320
HEIGHT  equ 200

start:
        cld
        push    cs
        pop     ds

        ; VGA 320x200x256
        mov     ax,0013h
        int     10h
        mov     ax,0A000h
        mov     es,ax

        xor     ax,ax
        mov     [time_base],al

main_loop:
        ; advance time (wrapping byte)
        mov     al,[time_base]
        add     al,2               ; tweak speed here
        mov     [time_base],al

        xor     di,di              ; VRAM ptr
        xor     bx,bx              ; y = 0

y_loop:
        mov     [ypos],bx
        xor     cx,cx              ; x = 0

x_loop:
        ; --- term1 = sin((x<<2) + t)
        mov     ax,cx
        shl     ax,2
        add     al,[time_base]     ; only low 8 bits matter
        movzx   si,al
        mov     al,[sinetab+si]
        movzx   bp,al              ; BP = s1 (0..255)

        ; --- term2 = sin((y<<2) + t*2)
        mov     ax,[ypos]
        shl     ax,2
        mov     dl,[time_base]
        shl     dl,1               ; t*2
        add     al,dl
        movzx   si,al
        mov     al,[sinetab+si]
        movzx   ax,al              ; zero-extend, AX=0..255
        add     bp,ax

        ; --- term3 = sin(((x+y)<<2) + t*3)
        mov     ax,cx
        add     ax,[ypos]
        shl     ax,2
        mov     dl,[time_base]
        mov     dh,dl
        shl     dl,1               ; dl = t*2
        add     dl,dh              ; dl = t*3
        add     al,dl
        movzx   si,al
        mov     al,[sinetab+si]
        movzx   ax,al
        add     bp,ax

        ; --- term4 = sin(((x-y)<<2) + t*4)
        mov     ax,cx
        sub     ax,[ypos]
        shl     ax,2
        mov     dl,[time_base]
        shl     dl,2               ; t*4
        add     al,dl
        movzx   si,al
        mov     al,[sinetab+si]
        movzx   ax,al
        add     bp,ax

        ; average 4 terms -> 0..255
        shr     bp,2
        mov     al,bl              ; AL = low byte of BP
        stosb

        inc     cx
        cmp     cx,WIDTH
        jb      x_loop

        inc     bx
        cmp     bx,HEIGHT
        jb      y_loop

        ; exit on key
        mov     ah,1
        int     16h
        jz      main_loop
        xor     ah,ah
        int     16h
        mov     ax,0003h
        int     10h
        mov     ax,4C00h
        int     21h

; ---------- data ----------
time_base   db 0
ypos        dw 0

; 256‑entry sine LUT: round( sin(2π*i/256)*127.5 + 127.5 )
sinetab:
        db 128,131,134,137,140,143,146,149,152,155,158,162,165,168,171,174
        db 177,180,183,186,189,191,194,197,200,202,205,207,210,212,215,217
        db 219,221,223,225,228,230,231,233,235,237,238,240,241,243,244,245
        db 246,247,248,249,250,251,251,252,252,253,253,253,254,254,254,254
        db 254,254,253,253,253,252,252,251,251,250,249,248,247,246,245,244
        db 243,241,240,238,237,235,233,231,230,228,225,223,221,219,217,215
        db 212,210,207,205,202,200,197,194,191,189,186,183,180,177,174,171
        db 168,165,162,158,155,152,149,146,143,140,137,134,131,128,125,122
        db 119,116,113,110,107,104,101, 97, 94, 91, 88, 85, 82, 79, 76, 73
        db  70, 67, 64, 62, 59, 56, 53, 51, 48, 46, 43, 41, 38, 36, 34, 32
        db  30, 28, 25, 23, 22, 20, 18, 16, 15, 13, 12, 10,  9,  8,  7,  6
        db   5,  4,  3,  2,  2,  1,  1,  0,  0,  0,  1,  1,  1,  1,  1,  1
        db   2,  2,  3,  4,  4,  5,  6,  7,  8, 10, 11, 13, 15, 17, 18, 20
        db  22, 24, 26, 28, 31, 33, 35, 37, 39, 41, 44, 46, 47, 49, 51, 53
        db  54, 56, 57, 59, 60, 61, 62, 63, 64, 65, 66, 66, 67, 67, 68, 68
        db  68, 68, 68, 68, 67, 67, 67, 66, 66, 65, 65, 64, 63, 6