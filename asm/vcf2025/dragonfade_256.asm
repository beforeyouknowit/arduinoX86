; Dragon Fade
; 256x256 version
; by HellMood
;
; set si to 100h

	  push 0xa000
    pop ds
S:
    shr al,1
    add dx,si
    sar dx,1
    jnc B
    or al,0x20
    add si,149
B:
    sub si,dx
    xor bl,bl
    mov bh,dl
    xor [bx+si],al
    jmp short S

    hlt