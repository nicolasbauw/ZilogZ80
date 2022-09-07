.target "z80"
.format "bin"

    LD  HL,0x1000
    LD  IX,0x1000
    LD  IY,0x1003
    SRL (HL)
    LD  A,(HL)
    SRL (IX+1)
    LD  A,(IX+1)
    SRL (IY-1)
    LD  A,(IY-1)