.target "z80"
.format "bin"

    LD  A,0x12
    LD  HL,0x1000
    LD  (HL),0x34
    RRD
    RLD
    LD  A,(HL)
    LD  A,0xFE
    LD  (HL),0x00
    RLD
    RRD
    LD  A,(HL)
    LD  A,0x01
    LD  (HL),0x00
    RLD
    RRD
    LD  A,(HL)
