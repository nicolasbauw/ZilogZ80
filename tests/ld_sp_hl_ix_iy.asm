.target "z80"
.format "bin"

    LD  HL,0x1234
    LD  IX,0x5678
    LD  IY,0x9ABC
    LD  SP,HL
    LD  SP,IX
    LD  SP,IY
