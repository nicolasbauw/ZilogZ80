.target "z80"
.format "bin"

    LD  A,0x00
    LD  IX,0x4161
    ADC A,A
    ADC A,IXH
    ADC A,IXL
