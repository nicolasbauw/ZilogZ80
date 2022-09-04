.target "z80"
.format "bin"

    SUB A
    CPL
    CPL
    ADD A,0xAA
    CPL
    CPL
