

cpu	8086
org	0h

    db 0xD6 ; SALC
    fnstcw [0]
    fwait
    nop
    nop
