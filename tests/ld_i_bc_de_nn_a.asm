.target "z80"
.format "bin"

    LD  BC,0x1000
    LD  DE,0x1001
    LD  A,0x77
    LD  (BC),A
    LD  (DE),A
    LD  (0x1002),A
