.target "z80"
.format "bin"

    LD  A,0x04
    LD  IX,0x01F8
    SUB A,A
    SBC A,IXH
    SBC A,IXL
    LD  A,0x04
    LD  IY,0x01F8
    SUB A,A
    SBC A,IYH
    SBC A,IYL
