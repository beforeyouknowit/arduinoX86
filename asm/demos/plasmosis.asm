; Plasmosis
; by Plex / BionFX
%define DOS 0

org 100h

%if DOS
  mov   al, 13h
  int   10h
%endif
  push  0xa000
  pop   es
  mov   bx, 320      ; line width

nextPixel:
  ; avg current value and 16-bit pixel in an adjacent line
  add   ax, word [es:di+bx]
  rcr   ax, 1
  dec   ax
  neg   bx          ; switch lines

  stosw             ; paint two pixels (high byte = main color / low byte = gradients)
  jmp nextPixel