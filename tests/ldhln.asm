.target "z80"
.format "bin"

    LD  HL,0x2000
    LD  (HL),0x33
    LD  HL,0x1000
    LD  (HL),0x65
