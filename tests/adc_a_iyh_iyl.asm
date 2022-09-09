.target "z80"
.format "bin"

    LD  A,0x00
    LD  IY,0x4161
    ADC A,A
    ADC A,IYH
    ADC A,IYL
