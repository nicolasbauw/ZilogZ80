.target "z80"
.format "bin"

    LD  IX,0x2000
    LD  (IX+2),0x33
    LD  (IX-2),0x11
    LD  IY,0x1000
    LD  (IY+1),0x22
    LD  (IY-1),0x44
