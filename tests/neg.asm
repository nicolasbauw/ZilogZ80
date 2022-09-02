.target "z80"
.format "bin"

    LD  A,0x01
    NEG
    ADD A,0x01
    NEG
    SUB A,0x80
    NEG
    ADD A,0x40
    NEG
    