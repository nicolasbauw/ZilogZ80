.target "z80"
.format "bin"

    LD  IX,0x1003
    LD  A,(IX+0)
    LD  B,(IX+1)
    LD  C,(IX+2)
    LD  D,(IX-1)
    LD  E,(IX-2)
    LD  H,(IX+3)
    LD  L,(IX-3)

    LD  IY,0x1004
    LD  A,(IY+0)
    LD  B,(IY+1)
    LD  C,(IY+2)
    LD  D,(IY-1)
    LD  E,(IY-2)
    LD  H,(IY+3)
    LD  L,(IY-3)
