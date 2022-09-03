.target "z80"
.format "bin"

    LD  HL,0x1000
    LD  IX,0x1000
    LD  IY,0x1003
    DEC (HL)
    INC (HL)
    INC (IX+1)
    DEC (IX+1)
    INC (IY-1)
    DEC (IY-1)
