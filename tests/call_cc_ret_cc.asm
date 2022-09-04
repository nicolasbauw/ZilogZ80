.target "z80"
.format "bin"
.org 0x0204

    SUB  A
    CALL NZ,l0
    CALL Z,l0
    ADD  A,0x01
    CALL Z,l1
    CALL NZ,l1
    RLCA
    CALL PE,l2
    CALL PO,l2
    SUB  0x03
    CALL P,l3
    CALL M,l3
    CALL NC,l4
    CALL C,l4
    RET
l0: RET  NZ
    RET  Z
l1: RET  Z
    RET  NZ
l2: RET  PE
    RET  PO
l3: RET  P
    RET  M
l4: RET  NC
    RET  C
