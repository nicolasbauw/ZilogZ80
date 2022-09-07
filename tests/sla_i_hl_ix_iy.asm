.target "z80"
.format "bin"

    LD  HL,0x1000
    LD  IX,0x1000
    LD  IY,0x1003
    SLA (HL)
    LD  A,(HL)
    SLA (IX+1)
    LD  A,(IX+1)
    SLA (IY-1)
    LD  A,(IY-1)
