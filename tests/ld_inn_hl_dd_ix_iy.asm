.target "z80"
.format "bin"

    LD HL,0x0201
    LD (0x1000),HL
    LD BC,0x1234
    LD (0x1002),BC
    LD DE,0x5678
    LD (0x1004),DE
    LD HL,0x9ABC
    LD (0x1006),HL
    LD SP,0x1368
    LD (0x1008),SP
    LD IX,0x4321
    LD (0x100A),IX
    LD IY,0x8765
    LD (0x100C),IY
