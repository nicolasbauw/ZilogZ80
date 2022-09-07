.target "z80"
.format "bin"

    LD  A,0x01
    LD  B,0x80
    LD  C,0xAA
    LD  D,0xFE
    LD  E,0x7F
    LD  H,0x11
    LD  L,0x00
    SRL A
    SRL B
    SRL C
    SRL D
    SRL E
    SRL H
    SRL L
