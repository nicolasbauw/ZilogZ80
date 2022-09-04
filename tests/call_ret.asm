.target "z80"
.format "bin"
.org 0x0204

    CALL l0
    CALL l0
l0: RET
