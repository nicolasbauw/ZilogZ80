.target "z80"
.format "bin"

.org    0
        LD  SP,0xFF00
        LD  A,0x0F
        JP  start

int     .org 0x0066
        LD  B,A
        RETN

start   .org 0x0070
        EI
@loop   CP  B
        JP  NZ,@loop
        RET
.end
