.target "z80"
.format "bin"

    LD      HL,0x1000
    LD      DE,0x2000
    LD      BC,0x0003
    LDIR
    LD      A,0x33
