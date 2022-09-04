.target "z80"
.format "bin"
.org 0x0204

         SUB    A
         JP     NZ,label0
         JP     Z,label0
         NOP
label0:  ADD    A,0x01
         JP     Z,label1
         JP     NZ,label1
         NOP
label1:  RLCA
         JP     PE,label2
         JP     PO,label2
         NOP
label2:  ADD    A,0xFD
         JP     P,label3
         JP     M,label3
         NOP
label3:  JP     NC,label4
         JP     C,label4
         NOP
label4:  NOP
