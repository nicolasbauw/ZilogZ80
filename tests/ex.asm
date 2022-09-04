.target "z80"
.format "bin"

    LD      HL,0x1234
    LD      DE,0x5678
    EX      DE,HL
    LD      A,0x11
    EX      AF,AF'
    LD      A,0x22
    EX      AF,AF'
    LD      BC,0x9ABC
    EXX
    LD      HL,0x1111
    LD      DE,0x2222
    LD      BC,0x3333
    EXX
    LD      SP,0x0100
    PUSH    DE
    EX      (SP),HL
    LD      IX,0x8899
    EX      (SP),IX
    LD      IY,0x6677
    EX      (SP),IY
