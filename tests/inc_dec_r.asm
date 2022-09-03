.target "z80"
.format "bin"

    LD  A,0x00
    LD  B,0xFF
    LD  C,0x0F
    LD  D,0x0E
    LD  E,0x7F
    LD  H,0x3E
    LD  L,0x23
    INC A
    DEC A
    INC B
    DEC B
    INC C
    DEC C
    INC D
    DEC D
    CP  0x01    // set carry flag (should be preserved)
    INC E
    DEC E
    INC H
    DEC H
    INC L
    DEC L
