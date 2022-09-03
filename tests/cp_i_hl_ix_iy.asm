.target "z80"
.format "bin"

    LD  HL,0x1000
    LD  IX,0x1000
    LD  IY,0x1003
    LD  A,0x41
    CP  (HL)
    CP  (IX+1)
    CP  (IY-1)
