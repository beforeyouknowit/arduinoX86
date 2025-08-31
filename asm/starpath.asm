; starpath.asm
; by HellMood/DSR
; https://www.pouet.net/prod.php?which=103622
; Compile with nasm to build program.bin for cpu_client
; nasm program.asm -o program.bin
cpu	386
org	100h

%define DOS 1

start:
    push    0xa000	  ; let ES point to 0xA000
    pop     es				; start of VGA Video RAM
    mov     al,0x13	  ; mode 13h, 320x200 pixels, 256 colors
%if DOS
    int     10h			  ; set graphic mode
%endif
X:
    mov     bl,14		  ; start depth at 14
L:
    mov     ax,0xcccd	; Rrrola constant
    mul     di				; Getting X,Y in DL,DH
    mov     al,dh			; getting Y into AL
    mul     bl				; multiply Y by current depth (into AH)
    xchg    ax,dx			; store Y' into DH, get X into AL
    sub     al,bl			; curve X by the current depth
    jc      W				  ; if left of the curve, jump to "sky"
    mul     bl				; multiply X by current depth (into AH)
    mov     al,dh			; get Y' in AL (now AL,AH = Y',X')
    or      al,ah			; OR for geometry and texture pattern
    lea     dx,[bx+si]		; get (current depth) + (current frame count) in DX (DL)
    and     ax,dx			; mask geometry/texture by time shifted depth...
    inc     bx				; (increment depth by one)
    test    al,16			; ... to create "gaps"
    jz      L				  ; if ray did not hit, repeat pixel loop
    jmp     short Q	  ; jump over the sky ^^
W:
    mov     al,27		  ; is both the star color and palette offset into sky
    add     dl,cl			; pseudorandom multiplication leftover DL added
    jz      Q				  ; to shifted depth, 1 in 256 chance to be a star *
    shld    ax,di,4		; if not, shift the starcolor and add scaled pixel count
Q:
    stosb			        ; write pixel to RAM, advance pixel counter
    inc     di				; increment pixel counter twice
    inc     di				; for a slightly smoother animation
    loop    X				  ; repeat for 64k pixels
    inc     si				; increment frame counter
%if DOS
    hlt					      ; synced against timer, wait (18.2 FPS)
    in      ax,0x60	  ; read keyboard
    dec     ax			  ; check for ESC
    jnz     X
%else
    jmp     X			    ; if not, repeat process
%endif
    ret					      ; quit program


