.target "z80"
.format "bin"

    LD  A,0x04
    LD  B,0x01
    LD  C,0xF8
    LD  D,0x0F
    LD  E,0x79
    LD  H,0xC0
    LD  L,0xBF
    SUB A,A
    SBC A,B
    SBC A,C
    SBC A,D
    SBC A,E
    SBC A,H
    SBC A,L
    SBC A,0x01
    SBC A,0xFE
    