.target "z80"
.format "bin"

    LD  A,0x04
    LD  IY,0x01F8
    SUB A,A
    SUB A,IYH
    SUB A,IYL
