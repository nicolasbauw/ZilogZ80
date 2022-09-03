.target "z80"
.format "bin"

    LD  A,0x04
    LD  B,0x05
    LD  C,0x03
    LD  D,0xff
    LD  E,0xaa
    LD  H,0x80
    LD  L,0x7f
    CP  A
    CP  B
    CP  C
    CP  D
    CP  E
    CP  H
    CP  L
    CP  0x04
