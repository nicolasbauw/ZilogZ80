.target "z80"
.format "bin"

    LD  A,0x0F
    ADD A,A
    LD  IY,0xE080
    ADD A,IYH
    LD  A,0x81
    ADD A,IYL
