.target "z80"
.format "bin"

    LD  A,0x01
    LD  B,0x80
    LD  C,0xAA
    LD  D,0xFE
    LD  E,0x7F
    LD  H,0x11
    LD  L,0x00
    SRA A
    SRA B
    SRA C
    SRA D
    SRA E
    SRA H
    SRA L
