.target "z80"
.format "bin"

    LD  HL,0x1000
    LD  IX,0x1000
    LD  IY,0x1003
    LD  A,0x00
    ADD A,(HL)
    ADC A,(IX+1)
    ADC A,(IY-1)
    ADC A,(IX+3)
