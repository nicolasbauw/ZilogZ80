.target "z80"
.format "bin"

.org    0
        LD  SP,0xFF00
        LD  A,0x0F
        JP  start

.org 0x0018
        LD  C,A
        RET

.org 0x0038
        LD  B,A
        RET

start   .org 0x0050
        IM  1
        EI
@loop   CP  B
        JP  NZ,@loop
        RET
.end
