.target "z80"
.format "bin"
.org 0x0204

     LD HL,l3
     LD IX,l4
     LD IY,l5
     JP l0
l1:  JR l2
l0:  JR l1
l3:  JP (IX)
l2:  JP (HL)
l4:  JP (IY)
l6:  JR l7
     NOP
     NOP
     NOP
     NOP
l5:  JR l6
l7:  NOP
