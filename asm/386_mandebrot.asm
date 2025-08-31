; mandel32.asm — 386 real-mode DOS VGA 13h Mandelbrot
; This code is public domain. Do whatever with it.
;
; This version has some basic mandelbrot optimizations.
;  - Vertical symmetry
;  - Fast interior tests (period-2 bulb + main cardioid)

        cpu     386
        bits    16
        org     100h


%define VCF_DEMO        1
%define DOS             0                   ; set to 0 for bare-metal (ArduinoX86, ROM, etc)
%define DIAG            0
%define COLOR_OFFSET    32                  ; signed imm8; palette rotation added to escape color

%if VCF_DEMO
  %define WIDTH           200               ; reduced width
  %define HEIGHT          160               ; reduced height
%else
  %define WIDTH           320               ; mode 13h width
  %define HEIGHT          200               ; mode 13h height
%endif
%define SCALE           65536               ; 16.16 fixed-point scale factor (1.0 = 65536)
%define ESCAPE_R2       (4*SCALE)           ; escape radius^2 = 4.0 in 16.16
%define STEP_XY         ((3*SCALE)/WIDTH)   ; per-pixel step in 16.16 for both axes
%define X_MIN           (-2*SCALE)          ; left edge of viewport (Cr start)
%define Y_MIN           (-(HEIGHT*STEP_XY)/2) ; center row is always Ci=0 for any WIDTH/HEIGHT

%define HALF_H          (HEIGHT/2)          ; 100
%define MAX_ITER        32                  ; iteration cap - any higher than 48 is invisible at 320x200
%define LAST_ROW_OFS    ((HEIGHT-1)*WIDTH)  ; offset to start of bottom row
%define CENTER_OFS      (HALF_H*WIDTH)      ; offset to start of center row

%if DIAG
    %define FAST_COLOR 4                    ; red = interior via fast tests (debug view)
%else
    %define FAST_COLOR 0                    ; black = interior (normal)
%endif

start:
%if DOS
        mov     ax, 0013h                   ; BIOS: set VGA 320x200x256
        int     10h
%endif
        push    cs                          ; DS=CS so we can read our data
        pop     ds
        mov     ax, 0A000h                  ; ES = VGA framebuffer segment
        mov     es, ax

        xor     di, di                      ; DI = current pixel pointer within ES (start of row 0)
        mov     word [rows_left], (HALF_H+1) ; render from top down through center row
        mov     dword [ci_cur], Y_MIN       ; Ci for current row (topmost starts at Y_MIN)

row_loop:
        mov     ax, di                      ; AX = current pixel offset
        add     ax, WIDTH                   ; AX = end offset for this row (one past last pixel)
        mov     [row_end], ax               ; store row end sentinel

        ; ci2_row = (ci*ci)>>16 (cache Ci^2 for this row)
        mov     eax, [ci_cur]               ; EAX = Ci (16.16)
        imul    eax                         ; EDX:EAX = Ci^2 (32.32)
        shrd    eax, edx, 16                ; EAX = Ci^2 >> 16 -> 16.16
        mov     [ci2_row], eax              ; save row’s Ci^2

        mov     ebx, X_MIN                  ; EBX = Cr, start at left edge for this row

pixel_loop:
        ; cache y^2 (16.16) for this pixel's tests
        mov     esi, [ci2_row]              ; ESI = Ci^2 (16.16) reused in fast tests

        ; -------- period-2 bulb test: (Cr+1)^2 + Ci^2 < 1/16 ? --------
        mov     eax, ebx                    ; EAX = Cr
        add     eax, (1*SCALE)              ; EAX = Cr + 1
        imul    eax                         ; EDX:EAX = (Cr+1)^2 (32.32)
        shrd    eax, edx, 16                ; EAX = (Cr+1)^2 (16.16)
        add     eax, esi                    ; EAX = (Cr+1)^2 + Ci^2
        cmp     eax, (SCALE/16)             ; compare with 0.0625
        jb      inside_fast                 ; inside 2-bulb -> fill FAST_COLOR

        ; -------- main cardioid test (signed 64-bit compare) --------
        ; xr' = Cr - 0.25
        mov     eax, ebx                    ; EAX = Cr
        sub     eax, (SCALE/4)              ; EAX = Cr - 0.25
        mov     ebp, eax                    ; EBP = xr' (kept for later add)

        imul    eax                         ; EDX:EAX = xr'^2 (32.32)
        mov     ecx, [ci2_row]              ; ECX = Ci^2 (16.16)
        mov     esi, ecx                    ; ESI = copy for splitting high/low parts
        shl     ecx, 16                     ; ECX.low32 = Ci^2.low16 <<16 (low 32 for 32.32 add)
        shr     esi, 16                     ; ESI.high32 = Ci^2.high16 (high 32 for 32.32 add)
        add     eax, ecx                    ; add low halves
        adc     edx, esi                    ; add high halves with carry

        add     eax, 8000h                  ; round 32.32 -> 16.16 by +0.5 ulp
        adc     edx, 0
        shrd    eax, edx, 16                ; EAX = q (16.16)

        mov     ecx, eax                    ; ECX = q (16.16)
        add     ecx, ebp                    ; ECX = q + xr'
        imul    ecx                         ; EDX:EAX = P64 = q*(q+xr') in 32.32

        ; RHS = (Ci^2)/4 + ε, promoted to 32.32 split
        mov     ecx, [ci2_row]              ; reload full Ci^2 (16.16)
        mov     ebp, ecx                    ; EBP = Ci^2
        shr     ebp, 2                      ; EBP = (Ci^2 / 4).16.16
        mov     esi, ebp                    ; ESI = copy for high half
        shr     esi, 16                     ; ESI = RHS.high32 (top 32 bits)
        shl     ebp, 16                     ; EBP = RHS.low32 (bottom 32 bits)
        add     ebp, 1                      ; add tiny epsilon to bias “<=”
        adc     esi, 0

        ; signed compare P64 <= RHS ?
        cmp     edx, esi                    ; compare high dwords (signed)
        jl      inside_fast                 ; P.high < RHS.high -> inside
        jg      not_inside_cardioid         ; P.high > RHS.high -> outside
        cmp     eax, ebp                    ; highs equal -> compare lows
        jbe     inside_fast                 ; P.low <= RHS.low -> inside

not_inside_cardioid:
        ; -------- slow path: iterate z_{n+1}=z_n^2 + c --------
        xor     esi, esi                    ; ESI = zr = 0.0
        xor     ebp, ebp                    ; EBP = zi = 0.0
        xor     ecx, ecx                    ; ECX = iter = 0
        push    di                          ; save pixel pointer for STOSB later

iter_loop:
        ; zr2 = (zr*zr) >> 16
        mov     eax, esi                    ; EAX = zr
        imul    eax                         ; EDX:EAX = zr^2 (32.32)
        shrd    eax, edx, 16                ; EAX = zr2 (16.16)
        mov     edi, eax                    ; EDI = zr2

        ; zi2 = (zi*zi) >> 16
        mov     eax, ebp                    ; EAX = zi
        imul    eax                         ; EDX:EAX = zi^2 (32.32)
        shrd    eax, edx, 16                ; EAX = zi2 (16.16)

        ; keep zr for cross term
        mov     edx, esi                    ; EDX = zr_old

        ; bailout if zr2 + zi2 > 4.0
        add     eax, edi                    ; EAX = zr2 + zi2
        cmp     eax, ESCAPE_R2              ; compare to 4.0 (16.16)
        ja      escaped_slow                ; escaped
        sub     eax, edi                    ; restore EAX = zi2 (for zr update)

        ; zr = zr2 - zi2 + Cr
        mov     esi, edi                    ; ESI = zr2
        sub     esi, eax                    ; zr2 - zi2
        add     esi, ebx                    ; + Cr

        ; zi = ((2*zr_old*zi_old) >> 15) + Ci
        mov     eax, edx                    ; EAX = zr_old
        imul    ebp                         ; EDX:EAX = zr_old * zi_old (32.32)
        shrd    eax, edx, 15                ; EAX = (2*zr*zi) >> 16 (one extra shift for factor 2)
        add     eax, [ci_cur]               ; + Ci
        mov     ebp, eax                    ; zi = result

        inc     ecx                         ; iter++
        cmp     ecx, MAX_ITER               ; reached cap?
        jb      iter_loop                   ; keep iterating

        ; in-set (didn’t escape by MAX_ITER)
        pop     di                          ; restore pixel pointer
        xor     ax, ax                      ; AX=0 -> color 0 (black)
        jmp     short store                 ; write pixel

inside_fast:
        mov     al, FAST_COLOR              ; AL = interior color from fast test
        jmp     short store_pre             ; go write pixel

escaped_slow:
        pop     di                          ; restore pixel pointer
        mov     al, cl                      ; AL = escape-time color (iteration count)
%if COLOR_OFFSET
        add     al, COLOR_OFFSET            ; optional palette rotation
%endif
        jmp     short store                 ; write pixel

store:
store_pre:
        stosb                               ; *ES:DI = AL; DI++
        add     ebx, STEP_XY                ; Cr += step to next pixel
        cmp     di, [row_end]               ; reached row end?
        jb      pixel_loop                  ; no: do next pixel

        ; mirror current row to bottom (except true center row)
        mov     ax, di                      ; AX = end-of-row pointer (start of next row)
        mov     si, ax                      ; SI = copy
        sub     si, WIDTH                   ; SI = start offset of the row we just finished

        mov     bx, LAST_ROW_OFS            ; BX = start of last row
        sub     bx, ax                      ; BX = last_row_start - end_ptr
        add     bx, WIDTH                   ; BX = start offset of destination row to mirror to

        cmp     si, CENTER_OFS              ; is this the center row?
        je      skip_mirror                 ; yes: don’t mirror it

        push    ds                          ; Save DS
        mov     dx, es
        mov     ds, dx                      ; DS := ES so MOVSD reads from video mem
        mov     di, bx                      ; DI = dest start (bottom half row)
        mov     cx, WIDTH/4                 ; copy 320 bytes as 80 dwords
        rep     movsd                       ; mirror top row into bottom
        pop     ds                          ; restore DS

skip_mirror:
        add     dword [ci_cur], STEP_XY     ; Ci += step (move down one row)
        mov     di, ax                      ; DI = start of next row (carry pointer forward)
        dec     word [rows_left]            ; rows left (top through center)?
        jnz     row_loop                    ; more rows -> loop

%if DOS
        xor     ah, ah                      ; wait for key (INT 16h, AH=0)
        int     16h
        mov     ax, 0003h                   ; text mode 80x25
        int     10h
        mov     ax, 4C00h                   ; exit to DOS
        int     21h
%else
.hang:  hlt                                 ; bare-metal hang if DOS=0
        jmp     .hang
%endif

; -------- data --------
section .bss
  ci_cur      dd  0                           ; current Ci (16.16) for the row
  ci2_row     dd  0                           ; cached Ci^2 (16.16) for the row
  row_end     dw  0                           ; offset one-past-end of current row
  rows_left   dw  0                           ; rows to render (top..center inclusive)