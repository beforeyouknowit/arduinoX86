cpu 386
    mov     eax, cr0
    or      eax, 1<<1              ; MP = 1 (monitor coprocessor)
    and     eax, ~((1<<2)|(1<<3))  ; EM = 0 (no emulation), TS = 0
    mov     cr0, eax
    fninit
    fnclex                          ; clear any pending exceptions, just in case
    fsave   [0200h]
    hlt