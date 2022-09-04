.target "z80"
.format "bin"

    LD HL,0X1000
    LD BC,0X0004
    LD A,0X03
    CPIR
    CPIR

