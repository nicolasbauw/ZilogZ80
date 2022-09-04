.target "z80"
.format "bin"

    LD  HL,0x1000 
    LD  A,0x12
    LD  (HL),A
    LD  B,0x13
    LD  (HL),B
    LD  C,0x14
    LD  (HL),C
    LD  D,0x15
    LD  (HL),D
    LD  E,0x16
    LD  (HL),E
    LD  (HL),H
    LD  (HL),L
