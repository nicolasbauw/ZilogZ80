.target "z80"
.format "bin"

    SUB A
    LD  B,0x01
    LD  C,0x03
    LD  D,0x07
    LD  E,0x0F
    LD  H,0x1F
    LD  L,0x3F
    XOR A
    XOR B
    XOR C
    XOR D
    XOR E
    XOR H
    XOR L
    XOR 0x7F
    XOR 0xFF