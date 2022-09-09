.target "z80"
.format "bin"

    LD  A,0x0F
    ADD A,A
    LD  IX,0xE080
    ADD A,IXH
    LD  A,0x81
    ADD A,IXL
