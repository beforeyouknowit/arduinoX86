;;
;; play with 70_000 cycles in dosbox
;;
%define DOS 1
org 100h

    les bp, [bx]      ; sets BP = 0x20CD --> so I have a decent start value for a count down to zero

%if DOS
    mov al, 0x13
    int 0x10
%endif
    shr bp, 1         ; half the duration until reaching zero

; palette
palette_loop:
    mov dx, 3C9h      ; data register for color palette (0x3C9)
    mov al, cl
    out dx, al        ; red
    shr al, 1
    out dx, al        ; green
    shr al, 1
    out dx, al        ; blue
    loop palette_loop

frameloop:
    mov ax,0cccdh
    mul di			; dl = x , dh=y

    sub dx, 140 + 100 * 256 ; center tunnels on screen

    ; calculate z^2 = RADIUS^2 - (x^2 + y^2)
    mov al, dl
    imul al         ; dx = x^2
    mov bx, ax
    mov al, dh
    imul al         ; dx = y^2
    add bx, ax      ; result in bx
    jz paint

    ; divide
    movsx ax, dl
    imul bp
    idiv bx

    add ax, bp
    xor al, 8
paint:
    stosb

    cmpsw           ; inc DI, 3 interlace effect
    loop frameloop

    dec bp          ; decrement time
    jmp  frameloop