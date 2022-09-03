.target "z80"
.format "bin"

    LD  HL,0x1000
    LD  IX,0x1000
    LD  IY,0x1003
    OR  (HL)
    OR  (IX+1)
    OR  (IY-1)
    XOR (HL)
    XOR (IX+1)
    XOR (IY-1)
