.target "z80"
.format "bin"

.org    0
        LD  SP,0xFF00
        LD  A,0x01
        LD  I,A
        LD  A,0x0F
        JP  start

.org    0x0038
        LD  D,A
        RET

start   .org    0x0050
        IM  2
        EI
@loop   CP  B
        JP  NZ,@loop
        RET

.org    0x0102
.byte   06

.org 0x0106
    LD  B,A
    RET
.end
