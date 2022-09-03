.target "z80"
.format "bin"

    LD  HL,0x1000
    LD  IX,0x1000
    LD  IY,0x1003
    LD  A,0xFF
    AND (HL)
    AND (IX+1)
    AND (IY-1)
