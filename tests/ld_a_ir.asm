.target "z80"
.format "bin"

    EI
    LD A,I
    SUB A
    LD A,R
