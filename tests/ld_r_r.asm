.target "z80"
.format "bin"

    LD  B,A
    LD  C,A
    LD  D,A
    LD  E,A
    LD  H,A
    LD  L,A
    LD  A,A

    LD  C,B
    LD  D,C
    LD  E,D
    LD  H,E
    LD  L,H
    LD  A,L
    