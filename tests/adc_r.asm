.target "z80"
.format "bin"

    LD  A,0x00
    LD  B,0x41
    LD  C,0x61
    LD  D,0x81
    LD  E,0x41
    LD  H,0x61
    LD  L,0x81
    ADC A,A
    ADC A,B
    ADC A,C
    ADC A,D
    ADC A,E
    ADC A,H
    ADC A,L
    ADC A,0x01
