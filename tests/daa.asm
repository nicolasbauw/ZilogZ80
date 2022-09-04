.target "z80"
.format "bin"

    LD  A,0x15
    LD  B,0x27
    ADD A,B
    DAA
    SUB B
    DAA
    LD  A,0x90
    LD  B,0x15
    ADD A,B
    DAA
    SUB B
    DAA
