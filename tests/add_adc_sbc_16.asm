.target "z80"
.format "bin"

    LD  HL,0x00FC
    LD  BC,0x0008
    LD  DE,0xFFFF
    ADD HL,BC
    ADD HL,DE
    ADC HL,BC
    ADD HL,HL
    ADD HL,DE
    SBC HL,BC
    LD  IX,0x00FC
    LD  SP,0x1000
    ADD IX, BC
    ADD IX, DE
    ADD IX, IX
    ADD IX, SP
    LD  IY,0xFFFF
    ADD IY,BC
    ADD IY,DE
    ADD IY,IY
    ADD IY,SP
    LD  HL,0x7FFF
    LD  BC,0x0001
    ADC HL,BC
    SBC HL,BC
