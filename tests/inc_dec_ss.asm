.target "z80"
.format "bin"

    LD  BC,0x0000
    LD  DE,0xffff
    LD  HL,0x00ff
    LD  SP,0x1111
    DEC BC
    INC BC
    INC DE
    DEC DE
    INC HL
    DEC HL
    INC SP
    DEC SP
