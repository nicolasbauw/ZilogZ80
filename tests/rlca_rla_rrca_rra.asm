.target "z80"
.format "bin"

    LD A,0xA0
    RLCA
    RLCA
    RRCA
    RRCA
    RLA
    RLA
    RRA
    RRA
