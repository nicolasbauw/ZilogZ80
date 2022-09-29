.target "z80"
.format "bin"
.org 0

        LD  SP,0xFF00
        LD  A,0xBB
        LD  B,0xFF

@loop   DEC B
        JP  NZ,@loop
        OUT (0x07),A
        RET
