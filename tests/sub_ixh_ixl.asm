.target "z80"
.format "bin"

    LD  A,0x04
    LD  IX,0x01F8
    SUB A,A
    SUB A,IXH
    SUB A,IXL
