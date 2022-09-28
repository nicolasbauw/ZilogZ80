.target "z80"
.format "bin"
.org 0

            LD  SP,0xFF00
            LD  B,0xDE
@loop       IN  A,(0x07)
            CP  B
            JP  NZ,@loop
            RET
