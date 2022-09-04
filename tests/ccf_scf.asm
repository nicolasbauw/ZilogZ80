.target "z80"
.format "bin"

    SUB A
    SCF
    CCF
    SUB 0xCC
    CCF
    SCF
