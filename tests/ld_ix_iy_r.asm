.target "z80"
.format "bin"

    LD  IX,0x1003
    LD  A,0x12
    LD  (IX+0),A
    LD  B,0x13
    LD  (IX+1),B
    LD  C,0x14
    LD  (IX+2),C
    LD  D,0x15
    LD  (IX-1),D
    LD  E,0x16
    LD  (IX-2),E
    LD  H,0x17
    LD  (IX+3),H
    LD  L,0x18
    LD  (IX-3),L
    LD  IY,0x1003
    LD  A,0x12
    LD  (IY+0),A
    LD  B,0x13
    LD  (IY+1),B
    LD  C,0x14
    LD  (IY+2),C
    LD  D,0x15
    LD  (IY-1),D
    LD  E,0x16
    LD  (IY-2),E
    LD  H,0x17
    LD  (IY+3),H
    LD  L,0x18
    LD  (IY-3),L