.target "z80"
.format "bin"

    SUB A
    JR  NZ, l0
    JR  Z, l0
    NOP
l0: ADD A,0x01
    JR  Z, l1
    JR  NZ, l1
    NOP
l1: SUB 0x03
    JR  NC, l2
    JR  C, l2
    NOP
l2: NOP
