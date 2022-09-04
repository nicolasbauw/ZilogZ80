.target "z80"
.format "bin"

    LD HL,0x1003
    LD BC,0x0004
    LD A,0x03
    CPD
    CPD
    CPD
    CPD
