.target "z80"
.format "bin"

    LD  HL,(0x1000)
    LD  BC,(0x1001)
    LD  DE,(0x1002)
    LD  HL,(0x1003)
    LD  SP,(0x1004)
    LD  IX,(0x1005)
    LD  IY,(0x1006)