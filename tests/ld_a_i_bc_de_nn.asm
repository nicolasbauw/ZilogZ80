.target "z80"
.format "bin"

    LD BC,0x1000
    LD DE,0x1001
    LD A,(BC)
    LD A,(DE)
    LD A,(0x1002)
