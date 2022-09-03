use crate::cpu::CPU;

// carry flag
const CF: u8 = 1 << 0;

// add/subtract flag
const NF: u8 = 1 << 1;

// overflow flag (same as parity)
const VF: u8 = 1 << 2;

// parity flag (same as overflow)
const PF: u8 = 1 << 2;

// undocumented 'X' flag
const XF: u8 = 1 << 3;

// half carry flag
const HF: u8 = 1 << 4;

// undocumented 'Y' flag
const YF: u8 = 1 << 5;

// zero flag
const ZF: u8 = 1 << 6;

// sign flag
const SF: u8 = 1 << 7;

#[test]
fn ld_r_r_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/ld_r_r.bin", 0).unwrap();
    c.registers.a = 0x12;
    assert_eq!(c.execute(),4); assert_eq!(c.registers.b, 0x12);     // LD B,A
    assert_eq!(c.execute(),4); assert_eq!(c.registers.c, 0x12);     // LD C,A
    assert_eq!(c.execute(),4); assert_eq!(c.registers.d, 0x12);     // LD D,A
    assert_eq!(c.execute(),4); assert_eq!(c.registers.e, 0x12);     // LD E,A
    assert_eq!(c.execute(),4); assert_eq!(c.registers.h, 0x12);     // LD H,A
    assert_eq!(c.execute(),4); assert_eq!(c.registers.l, 0x12);     // LD L,A
    assert_eq!(c.execute(),4); assert_eq!(c.registers.a, 0x12);     // LD A,A
    c.registers.b = 0x13;
    assert_eq!(c.execute(),4); assert_eq!(c.registers.c, 0x13);     // LD C,B
    assert_eq!(c.execute(),4); assert_eq!(c.registers.d, 0x13);     // LD D,C
    assert_eq!(c.execute(),4); assert_eq!(c.registers.e, 0x13);     // LD E,D 
    assert_eq!(c.execute(),4); assert_eq!(c.registers.h, 0x13);     // LD H,E
    assert_eq!(c.execute(),4); assert_eq!(c.registers.l, 0x13);     // LD L,H
    assert_eq!(c.execute(),4); assert_eq!(c.registers.a, 0x13);     // LD A,L
}

#[test]
fn ld_hl_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/ld_hl.bin", 0x0100).unwrap();
    c.registers.a = 0x33;
    c.registers.set_hl(0x1000);
    c.pc = 0x0100;
    assert_eq!(c.execute(), 7); assert_eq!(c.bus.read_byte(0x1000), 0x33);      // LD (HL),A
    assert_eq!(c.execute(), 7); assert_eq!(c.registers.b, 0x33);                // LD B,(HL)
    assert_eq!(c.execute(), 7); assert_eq!(c.registers.c, 0x33);                // LD C,(HL)
    assert_eq!(c.execute(), 7); assert_eq!(c.registers.d, 0x33);                // LD D,(HL)
    assert_eq!(c.execute(), 7); assert_eq!(c.registers.e, 0x33);                // LD E,(HL)
    assert_eq!(c.execute(), 7); assert_eq!(c.registers.h, 0x33);                // LD H,(HL)
}

#[test]
fn ld_hl_n_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/ld_hl_n.bin", 0).unwrap();
    assert_eq!(c.execute(), 10); assert_eq!(c.registers.get_hl(), 0x2000);      // LD HL,0x2000
    assert_eq!(c.execute(), 10); assert_eq!(c.bus.read_byte(0x2000), 0x33);     // LD (HL),0x33
    assert_eq!(c.execute(), 10); assert_eq!(c.registers.get_hl(), 0x1000);      // LD HL,0x1000
    assert_eq!(c.execute(), 10); assert_eq!(c.bus.read_byte(0x1000), 0x65);     // LD (HL),0x65
}

#[test]
fn ld_ix_iy_n_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/ld_ix_iy_n.bin", 0).unwrap();
    assert_eq!(c.execute(), 14); assert_eq!(c.ix, 0x2000);                      // LD IX,0x2000
    assert_eq!(c.execute(), 19); assert_eq!(0x33, c.bus.read_byte(0x2002));     // LD (IX+2),0x33
    assert_eq!(c.execute(), 19); assert_eq!(0x11, c.bus.read_byte(0x1FFE));     // LD (IX-2),0x11
    assert_eq!(c.execute(), 14); assert_eq!(0x1000, c.iy);                      // LD IY,0x1000
    assert_eq!(c.execute(), 19); assert_eq!(0x22, c.bus.read_byte(0x1001));     // LD (IY+1),0x22
    assert_eq!(c.execute(), 19); assert_eq!(0x44, c.bus.read_byte(0x0FFF));     // LD (IY-1),0x44
}

#[test]
fn ld_hl_dd_ix_iy_inn_asm() {
    let mut c = CPU::new();
    c.bus.write_byte(0x1000, 0x01);
    c.bus.write_byte(0x1001, 0x02);
    c.bus.write_byte(0x1002, 0x03);
    c.bus.write_byte(0x1003, 0x04);
    c.bus.write_byte(0x1004, 0x05);
    c.bus.write_byte(0x1005, 0x06);
    c.bus.write_byte(0x1006, 0x07);
    c.bus.write_byte(0x1007, 0x08);
    c.bus.load_bin("bin/ld_hl_dd_ix_iy_inn.bin", 0).unwrap();
    assert_eq!(c.execute(), 16); assert_eq!(0x0201, c.registers.get_hl());      // LD HL,(0x1000)
    assert_eq!(c.execute(), 20); assert_eq!(0x0302, c.registers.get_bc());      // LD BC,(0x1001)
    assert_eq!(c.execute(), 20); assert_eq!(0x0403, c.registers.get_de());      // LD DE,(0x1002)
    assert_eq!(c.execute(), 16); assert_eq!(0x0504, c.registers.get_hl());      // LD HL,(0x1003)
    assert_eq!(c.execute(), 20); assert_eq!(0x0605, c.sp);                      // LD SP,(0x1004)
    assert_eq!(c.execute(), 20); assert_eq!(0x0706, c.ix);                      // LD IX,(0x1004)
    assert_eq!(c.execute(), 20); assert_eq!(0x0807, c.iy);                      // LD IY,(0x1005)
}

#[test]
fn ld_ix_iy_nn_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/ld_ix_iy_nn.bin", 0).unwrap();
    assert_eq!(c.execute(), 10); assert_eq!(0x1234, c.registers.get_bc());      // LD BC,0x1234
    assert_eq!(c.execute(), 10); assert_eq!(0x5678, c.registers.get_de());      // LD DE,0x5678
    assert_eq!(c.execute(), 10); assert_eq!(0x9ABC, c.registers.get_hl());      // LD HL,0x9ABC
    assert_eq!(c.execute(), 10); assert_eq!(0x1368, c.sp);                      // LD SP,0x1368
    assert_eq!(c.execute(), 14); assert_eq!(0x4321, c.ix);                      // LD IX,0x4321
    assert_eq!(c.execute(), 14); assert_eq!(0x8765, c.iy);                      // LD IY,0x8765        
}

#[test]
fn ld_sp_hl_ix_iy_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/ld_sp_hl_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(), 10); assert_eq!(0x1234, c.registers.get_hl());      // LD HL,0x1234
    assert_eq!(c.execute(), 14); assert_eq!(0x5678, c.ix);                      // LD IX,0x5678
    assert_eq!(c.execute(), 14); assert_eq!(0x9ABC, c.iy);                      // LD IY,0x9ABC
    assert_eq!(c.execute(), 6); assert_eq!(0x1234, c.sp);                       // LD SP,HL
    assert_eq!(c.execute(), 10); assert_eq!(0x5678, c.sp);                      // LD SP,IX
    assert_eq!(c.execute(), 10); assert_eq!(0x9ABC, c.sp);                      // LD SP,IY
}

#[test]
fn ld_r_ix_iy_asm() {
    let mut c = CPU::new();
    c.bus.write_byte(0x1000, 0x01);
    c.bus.write_byte(0x1001, 0x02);
    c.bus.write_byte(0x1002, 0x03);
    c.bus.write_byte(0x1003, 0x04);
    c.bus.write_byte(0x1004, 0x05);
    c.bus.write_byte(0x1005, 0x06);
    c.bus.write_byte(0x1006, 0x07);
    c.bus.write_byte(0x1007, 0x08);
    c.bus.load_bin("bin/ld_r_ix_iy.bin", 0).unwrap();                           
    assert_eq!(c.execute(), 14); assert_eq!(0x1003, c.ix);                      // LD  IX,0x1003
    assert_eq!(c.execute(), 19); assert_eq!(4, c.registers.a);                  // LD  A,(IX+0)
    assert_eq!(c.execute(), 19); assert_eq!(5, c.registers.b);                  // LD  B,(IX+1)
    assert_eq!(c.execute(), 19); assert_eq!(6, c.registers.c);                  // LD  C,(IX+2)
    assert_eq!(c.execute(), 19); assert_eq!(3, c.registers.d);                  // LD  D,(IX-1)
    assert_eq!(c.execute(), 19); assert_eq!(2, c.registers.e);                  // LD  E,(IX-2)
    assert_eq!(c.execute(), 19); assert_eq!(7, c.registers.h);                  // LD  H,(IX+3)
    assert_eq!(c.execute(), 19); assert_eq!(1, c.registers.l);                  // LD  L,(IX-3)
    assert_eq!(c.execute(), 14); assert_eq!(0x1004, c.iy);                      // LD  IY,0x1004
    assert_eq!(c.execute(), 19); assert_eq!(5, c.registers.a);                  // LD  A,(IY+0)
    assert_eq!(c.execute(), 19); assert_eq!(6, c.registers.b);                  // LD  B,(IY+1)
    assert_eq!(c.execute(), 19); assert_eq!(7, c.registers.c);                  // LD  C,(IY+2)
    assert_eq!(c.execute(), 19); assert_eq!(4, c.registers.d);                  // LD  D,(IY-1)
    assert_eq!(c.execute(), 19); assert_eq!(3, c.registers.e);                  // LD  E,(IY-2)
    assert_eq!(c.execute(), 19); assert_eq!(8, c.registers.h);                  // LD  H,(IY+3)
    assert_eq!(c.execute(), 19); assert_eq!(2, c.registers.l);                  // LD  L,(IY-3)
}

#[test]
fn ld_ix_iy_r_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/ld_ix_iy_r.bin", 0).unwrap();
    assert_eq!(c.execute(), 14);    assert_eq!(0x1003, c.ix);
    assert_eq!(c.execute(), 7);     assert_eq!(0x12, c.registers.a);         
    assert_eq!(c.execute(), 19);    assert_eq!(0x12, c.bus.read_byte(0x1003));  
    assert_eq!(c.execute(), 7);     assert_eq!(0x13, c.registers.b);         
    assert_eq!(c.execute(), 19);    assert_eq!(0x13, c.bus.read_byte(0x1004));  
    assert_eq!(c.execute(), 7);     assert_eq!(0x14, c.registers.c);         
    assert_eq!(c.execute(), 19);    assert_eq!(0x14, c.bus.read_byte(0x1005));  
    assert_eq!(c.execute(), 7);     assert_eq!(0x15, c.registers.d);         
    assert_eq!(c.execute(), 19);    assert_eq!(0x15, c.bus.read_byte(0x1002));  
    assert_eq!(c.execute(), 7);     assert_eq!(0x16, c.registers.e);         
    assert_eq!(c.execute(), 19);    assert_eq!(0x16, c.bus.read_byte(0x1001));  
    assert_eq!(c.execute(), 7);     assert_eq!(0x17, c.registers.h);         
    assert_eq!(c.execute(), 19);    assert_eq!(0x17, c.bus.read_byte(0x1006));  
    assert_eq!(c.execute(), 7);     assert_eq!(0x18, c.registers.l);         
    assert_eq!(c.execute(), 19);    assert_eq!(0x18, c.bus.read_byte(0x1000));  
    assert_eq!(c.execute(), 14);    assert_eq!(0x1003, c.iy);
    assert_eq!(c.execute(), 7);     assert_eq!(0x12, c.registers.a);        
    assert_eq!(c.execute(), 19);    assert_eq!(0x12, c.bus.read_byte(0x1003)); 
    assert_eq!(c.execute(), 7);     assert_eq!(0x13, c.registers.b);        
    assert_eq!(c.execute(), 19);    assert_eq!(0x13, c.bus.read_byte(0x1004)); 
    assert_eq!(c.execute(), 7);     assert_eq!(0x14, c.registers.c);        
    assert_eq!(c.execute(), 19);    assert_eq!(0x14, c.bus.read_byte(0x1005)); 
    assert_eq!(c.execute(), 7);     assert_eq!(0x15, c.registers.d);        
    assert_eq!(c.execute(), 19);    assert_eq!(0x15, c.bus.read_byte(0x1002)); 
    assert_eq!(c.execute(), 7);     assert_eq!(0x16, c.registers.e);        
    assert_eq!(c.execute(), 19);    assert_eq!(0x16, c.bus.read_byte(0x1001)); 
    assert_eq!(c.execute(), 7);     assert_eq!(0x17, c.registers.h);        
    assert_eq!(c.execute(), 19);    assert_eq!(0x17, c.bus.read_byte(0x1006)); 
    assert_eq!(c.execute(), 7);     assert_eq!(0x18, c.registers.l);        
    assert_eq!(c.execute(), 19);    assert_eq!(0x18, c.bus.read_byte(0x1000)); 
}

#[test]
fn push_pop_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/push_pop.bin", 0).unwrap();
    assert_eq!(c.execute(), 10); assert_eq!(0x1234, c.registers.get_bc());                                  // LD BC,0x1234
    assert_eq!(c.execute(), 10); assert_eq!(0x5678, c.registers.get_de());                                  // LD DE,0x5678
    assert_eq!(c.execute(), 10); assert_eq!(0x9ABC, c.registers.get_hl());                                  // LD HL,0x9ABC
    assert_eq!(c.execute(), 7);  assert_eq!(0xEF00, c.registers.get_af());                                  // LD A,0xEF
    assert_eq!(c.execute(), 14); assert_eq!(0x2345, c.ix);                                                  // LD IX,0x2345
    assert_eq!(c.execute(), 14); assert_eq!(0x6789, c.iy);                                                  // LD IY,0x6789
    assert_eq!(c.execute(), 10); assert_eq!(0x0100, c.sp);                                                  // LD SP,0x0100
    assert_eq!(c.execute(), 11); assert_eq!(0xEF00, c.bus.read_word(0x00FE)); assert_eq!(0x00FE, c.sp);     // PUSH AF
    assert_eq!(c.execute(), 11); assert_eq!(0x1234, c.bus.read_word(0x00FC)); assert_eq!(0x00FC, c.sp);     // PUSH BC
    assert_eq!(c.execute(), 11); assert_eq!(0x5678, c.bus.read_word(0x00FA)); assert_eq!(0x00FA, c.sp);     // PUSH DE
    assert_eq!(c.execute(), 11); assert_eq!(0x9ABC, c.bus.read_word(0x00F8)); assert_eq!(0x00F8, c.sp);     // PUSH HL
    assert_eq!(c.execute(), 15); assert_eq!(0x2345, c.bus.read_word(0x00F6)); assert_eq!(0x00F6, c.sp);     // PUSH IX
    assert_eq!(c.execute(), 15); assert_eq!(0x6789, c.bus.read_word(0x00F4)); assert_eq!(0x00F4, c.sp);     // PUSH IY
    assert_eq!(c.execute(), 10); assert_eq!(0x6789, c.registers.get_af()); assert_eq!(0x00F6, c.sp);        // POP AF
    assert_eq!(c.execute(), 10); assert_eq!(0x2345, c.registers.get_bc()); assert_eq!(0x00F8, c.sp);        // POP BC
    assert_eq!(c.execute(), 10); assert_eq!(0x9ABC, c.registers.get_de()); assert_eq!(0x00FA, c.sp);        // POP DE
    assert_eq!(c.execute(), 10); assert_eq!(0x5678, c.registers.get_hl()); assert_eq!(0x00FC, c.sp);        // POP HL
    assert_eq!(c.execute(), 14); assert_eq!(0x1234, c.ix); assert_eq!(0x00FE, c.sp);                        // POP IX
    assert_eq!(c.execute(), 14); assert_eq!(0xEF00, c.iy); assert_eq!(0x0100, c.sp);                        // POP IY
}

#[test]
fn add_r_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/add_r.bin", 0).unwrap();
    assert_eq!(c.execute(), 7); assert_eq!(0x0F, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);        // LD A,0x0F
    assert_eq!(c.execute(), 4); assert_eq!(0x1E, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF);       // ADD A,A
    assert_eq!(c.execute(), 7); assert_eq!(0xE0, c.registers.b);                                                    // LD B,0xE0
    assert_eq!(c.execute(), 4); assert_eq!(0xFE, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF);       // ADD A,B
    assert_eq!(c.execute(), 7); assert_eq!(0x81, c.registers.a);                                                    // LD A,0x81
    assert_eq!(c.execute(), 7); assert_eq!(0x80, c.registers.c);                                                    // LD C,0x80
    assert_eq!(c.execute(), 4); assert_eq!(0x01, c.registers.a); assert_eq!(c.registers.flags.to_byte(), VF|CF);    // ADD A,C
    assert_eq!(c.execute(), 7); assert_eq!(0xFF, c.registers.d);                                                    // LD D,0xFF
    assert_eq!(c.execute(), 4); assert_eq!(0x00, c.registers.a); assert_eq!(c.registers.flags.to_byte(), ZF|HF|CF); // ADD A,D
    assert_eq!(c.execute(), 7); assert_eq!(0x40, c.registers.e);                                                    // LD E,0x40
    assert_eq!(c.execute(), 4); assert_eq!(0x40, c.registers.a); assert_eq!(c.registers.flags.to_byte() , 0);       // ADD A,E
    assert_eq!(c.execute(), 7); assert_eq!(0x80, c.registers.h);                                                    // LD H,0x80
    assert_eq!(c.execute(), 4); assert_eq!(0xC0, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF);       // ADD A,H
    assert_eq!(c.execute(), 7); assert_eq!(0x33, c.registers.l);                                                    // LD L,0x33
    assert_eq!(c.execute(), 4); assert_eq!(0xF3, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF);       // ADD A,L
    assert_eq!(c.execute(), 7); assert_eq!(0x37, c.registers.a); assert_eq!(c.registers.flags.to_byte(), CF);       // ADD A,0x44  
}

#[test]
fn add_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    c.bus.write_byte(0x1000, 0x41);
    c.bus.write_byte(0x1001, 0x61);
    c.bus.write_byte(0x1002, 0x81);
    c.bus.load_bin("bin/add_i_hl_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(), 10); assert_eq!(0x1000, c.registers.get_hl());                                          // LD HL,0x1000
    assert_eq!(c.execute(), 14); assert_eq!(0x1000, c.ix);                                                          // LD IX,0x1000
    assert_eq!(c.execute(), 14); assert_eq!(0x1003, c.iy);                                                          // LD IY,0x1003
    assert_eq!(c.execute(), 7); assert_eq!(0x00, c.registers.a);                                                    // LD A,0x00
    assert_eq!(c.execute(), 7); assert_eq!(0x41, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);        // ADD A,(HL)
    assert_eq!(c.execute(), 19); assert_eq!(0xA2, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|VF);   // ADD A,(IX+1)
    assert_eq!(c.execute(), 19); assert_eq!(0x23, c.registers.a); assert_eq!(c.registers.flags.to_byte(), VF|CF);   // ADD A,(IY-1)
}

#[test]
fn adc_r_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/adc_r.bin", 0).unwrap();
    assert_eq!(c.execute(), 7); assert_eq!(0x00, c.registers.a);                                                    // LD A,0x00
    assert_eq!(c.execute(), 7); assert_eq!(0x41, c.registers.b);                                                    // LD B,0x41
    assert_eq!(c.execute(), 7); assert_eq!(0x61, c.registers.c);                                                    // LD C,0x61
    assert_eq!(c.execute(), 7); assert_eq!(0x81, c.registers.d);                                                    // LD D,0x81
    assert_eq!(c.execute(), 7); assert_eq!(0x41, c.registers.e);                                                    // LD E,0x41
    assert_eq!(c.execute(), 7); assert_eq!(0x61, c.registers.h);                                                    // LD H,0x61
    assert_eq!(c.execute(), 7); assert_eq!(0x81, c.registers.l);                                                    // LD L,0x81
    assert_eq!(c.execute(), 4); assert_eq!(0x00, c.registers.a); assert_eq!(c.registers.flags.to_byte(), ZF);       // ADC A,A
    assert_eq!(c.execute(), 4); assert_eq!(0x41, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);        // ADC A,B
    assert_eq!(c.execute(), 4); assert_eq!(0xA2, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|VF);    // ADC A,C
    assert_eq!(c.execute(), 4); assert_eq!(0x23, c.registers.a); assert_eq!(c.registers.flags.to_byte(), VF|CF);    // ADC A,D
    assert_eq!(c.execute(), 4); assert_eq!(0x65, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);        // ADC A,E
    assert_eq!(c.execute(), 4); assert_eq!(0xC6, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|VF);    // ADC A,H
    assert_eq!(c.execute(), 4); assert_eq!(0x47, c.registers.a); assert_eq!(c.registers.flags.to_byte(), VF|CF);    // ADC A,L
    assert_eq!(c.execute(), 7); assert_eq!(0x49, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);        // ADC A,0x01
}

#[test]
fn adc_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    c.bus.write_byte(0x1000, 0x41);
    c.bus.write_byte(0x1001, 0x61);
    c.bus.write_byte(0x1002, 0x81);
    c.bus.write_byte(0x1003, 0x02);
    c.bus.load_bin("bin/adc_i_hl_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(), 10); assert_eq!(0x1000, c.registers.get_hl());                                          // LD HL,0x1000
    assert_eq!(c.execute(), 14); assert_eq!(0x1000, c.ix);                                                          // LD IX,0x1000
    assert_eq!(c.execute(), 14); assert_eq!(0x1003, c.iy);                                                          // LD IY,0x1003
    assert_eq!(c.execute(), 7);  assert_eq!(0x00, c.registers.a);                                                   // LD A,0x00
    assert_eq!(c.execute(), 7);  assert_eq!(0x41, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);       // ADD A,(HL)
    assert_eq!(c.execute(), 19); assert_eq!(0xA2, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|VF);   // ADC A,(IX+1)
    assert_eq!(c.execute(), 19); assert_eq!(0x23, c.registers.a); assert_eq!(c.registers.flags.to_byte(), VF|CF);   // ADC A,(IY-1)
    assert_eq!(c.execute(), 19); assert_eq!(0x26, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);       // ADC A,(IX+3)
}

#[test]
fn sub_r_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/sub_r.bin", 0).unwrap();
    assert_eq!(c.execute(), 7); assert_eq!(0x04, c.registers.a);                                                        // LD A,0x04
    assert_eq!(c.execute(), 7); assert_eq!(0x01, c.registers.b);                                                        // LD B,0x01
    assert_eq!(c.execute(), 7); assert_eq!(0xF8, c.registers.c);                                                        // LD C,0xF8
    assert_eq!(c.execute(), 7); assert_eq!(0x0F, c.registers.d);                                                        // LD D,0x0F
    assert_eq!(c.execute(), 7); assert_eq!(0x79, c.registers.e);                                                        // LD E,0x79
    assert_eq!(c.execute(), 7); assert_eq!(0xC0, c.registers.h);                                                        // LD H,0xC0
    assert_eq!(c.execute(), 7); assert_eq!(0xBF, c.registers.l);                                                        // LD L,0xBF
    assert_eq!(c.execute(), 4); assert_eq!(0x00, c.registers.a); assert_eq!(c.registers.flags.to_byte(), ZF|NF);        // SUB A,A
    assert_eq!(c.execute(), 4); assert_eq!(0xFF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF);  // SUB A,B
    assert_eq!(c.execute(), 4); assert_eq!(0x07, c.registers.a); assert_eq!(c.registers.flags.to_byte(), NF);           // SUB A,C
    assert_eq!(c.execute(), 4); assert_eq!(0xF8, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF);  // SUB A,D
    assert_eq!(c.execute(), 4); assert_eq!(0x7F, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF|VF|NF);     // SUB A,E
    assert_eq!(c.execute(), 4); assert_eq!(0xBF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|VF|NF|CF);  // SUB A,H
    assert_eq!(c.execute(), 4); assert_eq!(0x00, c.registers.a); assert_eq!(c.registers.flags.to_byte(), ZF|NF);        // SUB A,L
    assert_eq!(c.execute(), 7); assert_eq!(0xFF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF);  // SUB A,0x01
    assert_eq!(c.execute(), 7); assert_eq!(0x01, c.registers.a); assert_eq!(c.registers.flags.to_byte(), NF);           // SUB A,0xFE
}

#[test]
fn cp_r_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/cp_r.bin", 0).unwrap();
    assert_eq!(c.execute(), 7); assert_eq!(0x04, c.registers.a);                                                        // LD A,0x04
    assert_eq!(c.execute(), 7); assert_eq!(0x05, c.registers.b);                                                        // LD B,0x05
    assert_eq!(c.execute(), 7); assert_eq!(0x03, c.registers.c);                                                        // LD C,0x03
    assert_eq!(c.execute(), 7); assert_eq!(0xff, c.registers.d);                                                        // LD D,0xff
    assert_eq!(c.execute(), 7); assert_eq!(0xaa, c.registers.e);                                                        // LD E,0xaa
    assert_eq!(c.execute(), 7); assert_eq!(0x80, c.registers.h);                                                        // LD H,0x80
    assert_eq!(c.execute(), 7); assert_eq!(0x7f, c.registers.l);                                                        // LD L,0x7f
    assert_eq!(c.execute(), 4); assert_eq!(0x04, c.registers.a); assert_eq!(c.registers.flags.to_byte(), ZF|NF);        // CP A
    assert_eq!(c.execute(), 4); assert_eq!(0x04, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF);  // CP B
    assert_eq!(c.execute(), 4); assert_eq!(0x04, c.registers.a); assert_eq!(c.registers.flags.to_byte(), NF);           // CP C
    assert_eq!(c.execute(), 4); assert_eq!(0x04, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF|NF|CF);     // CP D
    assert_eq!(c.execute(), 4); assert_eq!(0x04, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF|NF|CF);     // CP E
    assert_eq!(c.execute(), 4); assert_eq!(0x04, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|VF|NF|CF);  // CP H
    assert_eq!(c.execute(), 4); assert_eq!(0x04, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF);  // CP L
    assert_eq!(c.execute(), 7); assert_eq!(0x04, c.registers.a); assert_eq!(c.registers.flags.to_byte(), ZF|NF);        // CP 0x04
}

#[test]
fn sub_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    c.bus.write_byte(0x1000, 0x41);
    c.bus.write_byte(0x1001, 0x61);
    c.bus.write_byte(0x1002, 0x81);
    c.bus.load_bin("bin/sub_i_hl_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(), 10); assert_eq!(0x1000, c.registers.get_hl());                                              // LD HL,0x1000
    assert_eq!(c.execute(), 14); assert_eq!(0x1000, c.ix);                                                              // LD IX,0x1000
    assert_eq!(c.execute(), 14); assert_eq!(0x1003, c.iy);                                                              // LD IY,0x1003
    assert_eq!(c.execute(), 7);  assert_eq!(0x00, c.registers.a);                                                       // LD A,0x00
    assert_eq!(c.execute(), 7);  assert_eq!(0xBF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF); // SUB A,(HL)
    assert_eq!(c.execute(), 19); assert_eq!(0x5E, c.registers.a); assert_eq!(c.registers.flags.to_byte(), VF|NF);       // SUB A,(IX+1)
    assert_eq!(c.execute(), 19); assert_eq!(0xFD, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|NF|CF);    // SUB A,(IY-2)
}

#[test]
fn cp_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    c.bus.write_byte(0x1000, 0x41);
    c.bus.write_byte(0x1001, 0x61);
    c.bus.write_byte(0x1002, 0x22);
    c.bus.load_bin("bin/cp_i_hl_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(), 10); assert_eq!(0x1000, c.registers.get_hl());                                              // LD HL,0x1000
    assert_eq!(c.execute(), 14); assert_eq!(0x1000, c.ix);                                                              // LD IX,0x1000
    assert_eq!(c.execute(), 14); assert_eq!(0x1003, c.iy);                                                              // LD IY,0x1003
    assert_eq!(c.execute(), 7);  assert_eq!(0x41, c.registers.a);                                                       // LD A,0x41
    assert_eq!(c.execute(), 7);  assert_eq!(0x41, c.registers.a); assert_eq!(c.registers.flags.to_byte(), ZF|NF);       // CP (HL)
    assert_eq!(c.execute(), 19); assert_eq!(0x41, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|NF|CF);    // CP (IX+1)
    assert_eq!(c.execute(), 19); assert_eq!(0x41, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF|NF);       // CP (IY-1)
}

#[test]
fn sbc_r_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/sbc_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute();
    }
    assert_eq!(c.execute(), 4); assert_eq!(0x00, c.registers.a); assert_eq!(c.registers.flags.to_byte(), ZF|NF);        // SUB A,A
    assert_eq!(c.execute(), 4); assert_eq!(0xFF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF);  // SBC A,B
    assert_eq!(c.execute(), 4); assert_eq!(0x06, c.registers.a); assert_eq!(c.registers.flags.to_byte(), NF);           // SBC A,C
    assert_eq!(c.execute(), 4); assert_eq!(0xF7, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF);  // SBC A,D
    assert_eq!(c.execute(), 4); assert_eq!(0x7D, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF|VF|NF);     // SBC A,E
    assert_eq!(c.execute(), 4); assert_eq!(0xBD, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|VF|NF|CF);  // SBC A,H
    assert_eq!(c.execute(), 4); assert_eq!(0xFD, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF);  // SBC A,L
    assert_eq!(c.execute(), 7); assert_eq!(0xFB, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|NF);        // SBC A,0x01
    assert_eq!(c.execute(), 7); assert_eq!(0xFD, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF);  // SBC A,0xFE
}

#[test]
fn sbc_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    c.bus.write_byte(0x1000, 0x41);
    c.bus.write_byte(0x1001, 0x61);
    c.bus.write_byte(0x1002, 0x81);
    c.bus.load_bin("bin/sbc_i_hl_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(), 10); assert_eq!(0x1000, c.registers.get_hl());
    assert_eq!(c.execute(), 14); assert_eq!(0x1000, c.ix);
    assert_eq!(c.execute(), 14); assert_eq!(0x1003, c.iy);
    assert_eq!(c.execute(), 7);  assert_eq!(0x00, c.registers.a);
    assert_eq!(c.execute(), 7);  assert_eq!(0xBF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF);
    assert_eq!(c.execute(), 19); assert_eq!(0x5D, c.registers.a); assert_eq!(c.registers.flags.to_byte(), VF|NF);
    assert_eq!(c.execute(), 19); assert_eq!(0xFC, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|NF|CF);
}

#[test]
fn or_r_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/or_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute();
    }
    assert_eq!(c.execute(), 4); assert_eq!(0x00, c.registers.a); assert_eq!(c.registers.flags.to_byte(), ZF|PF);      // OR A
    assert_eq!(c.execute(), 4); assert_eq!(0x01, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);          // OR B
    assert_eq!(c.execute(), 4); assert_eq!(0x03, c.registers.a); assert_eq!(c.registers.flags.to_byte(), PF);         // OR C
    assert_eq!(c.execute(), 4); assert_eq!(0x07, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);          // OR D
    assert_eq!(c.execute(), 4); assert_eq!(0x0F, c.registers.a); assert_eq!(c.registers.flags.to_byte(), PF);         // OR E
    assert_eq!(c.execute(), 4); assert_eq!(0x1F, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);          // OR H
    assert_eq!(c.execute(), 4); assert_eq!(0x3F, c.registers.a); assert_eq!(c.registers.flags.to_byte(), PF);         // OR L
    assert_eq!(c.execute(), 7); assert_eq!(0x7F, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);          // OR 0x40
    assert_eq!(c.execute(), 7); assert_eq!(0xFF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|PF);      // OR 0x80
}

#[test]
fn xor_r_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/xor_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute();
    }
    assert_eq!(c.execute(), 4); assert_eq!(0x00, c.registers.a); assert_eq!(c.registers.flags.to_byte(), ZF|PF);    // XOR A
    assert_eq!(c.execute(), 4); assert_eq!(0x01, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);        // XOR B
    assert_eq!(c.execute(), 4); assert_eq!(0x02, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);        // XOR C
    assert_eq!(c.execute(), 4); assert_eq!(0x05, c.registers.a); assert_eq!(c.registers.flags.to_byte(), PF);       // XOR D
    assert_eq!(c.execute(), 4); assert_eq!(0x0A, c.registers.a); assert_eq!(c.registers.flags.to_byte(), PF);       // XOR E
    assert_eq!(c.execute(), 4); assert_eq!(0x15, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);        // XOR H
    assert_eq!(c.execute(), 4); assert_eq!(0x2A, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);        // XOR L
    assert_eq!(c.execute(), 7); assert_eq!(0x55, c.registers.a); assert_eq!(c.registers.flags.to_byte(), PF);       // XOR 0x7F
    assert_eq!(c.execute(), 7); assert_eq!(0xAA, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|PF);    // XOR 0xFF
}

#[test]
fn or_xor_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    c.bus.write_byte(0x1000, 0x41);
    c.bus.write_byte(0x1001, 0x62);
    c.bus.write_byte(0x1002, 0x84);
    c.bus.load_bin("bin/or_xor_i_hl_ix_iy.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute();
    }
    assert_eq!(c.execute(), 7);  assert_eq!(0x41, c.registers.a); assert_eq!(c.registers.flags.to_byte(), PF);      // OR (HL)
    assert_eq!(c.execute(), 19); assert_eq!(0x63, c.registers.a); assert_eq!(c.registers.flags.to_byte(), PF);      // OR (IX+1)
    assert_eq!(c.execute(), 19); assert_eq!(0xE7, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|PF);   // OR (IY-1)
    assert_eq!(c.execute(), 7);  assert_eq!(0xA6, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|PF);   // XOR (HL)
    assert_eq!(c.execute(), 19); assert_eq!(0xC4, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF);      // XOR (IX+1)
    assert_eq!(c.execute(), 19); assert_eq!(0x40, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);       // XOR (IY-1)
}

#[test]
fn and_r_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/and_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute();
    }
    assert_eq!(c.execute(), 4); assert_eq!(0x01, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF);          // AND B
    assert_eq!(c.execute(), 7); assert_eq!(0xFF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|PF);       // OR 0xFF
    assert_eq!(c.execute(), 4); assert_eq!(0x03, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF|PF);       // AND C
    assert_eq!(c.execute(), 7); assert_eq!(0xFF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|PF);       // OR 0xFF
    assert_eq!(c.execute(), 4); assert_eq!(0x04, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF);          // AND D
    assert_eq!(c.execute(), 7); assert_eq!(0xFF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|PF);       // OR 0xFF
    assert_eq!(c.execute(), 4); assert_eq!(0x08, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF);          // AND E
    assert_eq!(c.execute(), 7); assert_eq!(0xFF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|PF);       // OR 0xFF
    assert_eq!(c.execute(), 4); assert_eq!(0x10, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF);          // AND H
    assert_eq!(c.execute(), 7); assert_eq!(0xFF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|PF);       // OR 0xFF
    assert_eq!(c.execute(), 4); assert_eq!(0x20, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF);          // AND L
    assert_eq!(c.execute(), 7); assert_eq!(0xFF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|PF);       // OR 0xFF
    assert_eq!(c.execute(), 7); assert_eq!(0x40, c.registers.a); assert_eq!(c.registers.flags.to_byte(), HF);          // AND 0x40
    assert_eq!(c.execute(), 7); assert_eq!(0xFF, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|PF);       // OR 0xFF
    assert_eq!(c.execute(), 7); assert_eq!(0xAA, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|PF);    // AND 0xAA
}

#[test]
fn and_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    c.bus.write_byte(0x1000, 0xFE);
    c.bus.write_byte(0x1001, 0xAA);
    c.bus.write_byte(0x1002, 0x99);
    c.bus.load_bin("bin/and_i_hl_ix_iy.bin", 0).unwrap();
    for _ in 0..4 {
        c.execute();
    }
    assert_eq!(c.execute(), 7);  assert_eq!(0xFE, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF);       // AND (HL)
    assert_eq!(c.execute(), 19); assert_eq!(0xAA, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|PF);    // AND (IX+1)
    assert_eq!(c.execute(), 19); assert_eq!(0x88, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|PF);    // AND (IY-1)
}

#[test]
fn inc_dec_r_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/inc_dec_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute();
    }
    assert_eq!(c.execute(), 4); assert_eq!(0x01, c.registers.a); assert_eq!(c.registers.flags.to_byte(), 0);            // INC A
    assert_eq!(c.execute(), 4); assert_eq!(0x00, c.registers.a); assert_eq!(c.registers.flags.to_byte(), ZF|NF);        // DEC A
    assert_eq!(c.execute(), 4); assert_eq!(0x00, c.registers.b); assert_eq!(c.registers.flags.to_byte(), ZF|HF);        // INC B
    assert_eq!(c.execute(), 4); assert_eq!(0xFF, c.registers.b); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF);     // DEC B
    assert_eq!(c.execute(), 4); assert_eq!(0x10, c.registers.c); assert_eq!(c.registers.flags.to_byte(), HF);           // INC C
    assert_eq!(c.execute(), 4); assert_eq!(0x0F, c.registers.c); assert_eq!(c.registers.flags.to_byte(), HF|NF);        // DEC C
    assert_eq!(c.execute(), 4); assert_eq!(0x0F, c.registers.d); assert_eq!(c.registers.flags.to_byte(), 0);            // INC D
    assert_eq!(c.execute(), 4); assert_eq!(0x0E, c.registers.d); assert_eq!(c.registers.flags.to_byte(), NF);           // DEC D
    assert_eq!(c.execute(), 7); assert_eq!(0x00, c.registers.a); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF);  // CP 0x01   set carry flag (should be preserved)
    assert_eq!(c.execute(), 4); assert_eq!(0x80, c.registers.e); assert_eq!(c.registers.flags.to_byte(), SF|HF|VF|CF);  // INC E
    assert_eq!(c.execute(), 4); assert_eq!(0x7F, c.registers.e); assert_eq!(c.registers.flags.to_byte(), HF|VF|NF|CF);  // DEC E
    assert_eq!(c.execute(), 4); assert_eq!(0x3F, c.registers.h); assert_eq!(c.registers.flags.to_byte(), CF);           // INC H
    assert_eq!(c.execute(), 4); assert_eq!(0x3E, c.registers.h); assert_eq!(c.registers.flags.to_byte(), NF|CF);        // DEC H
    assert_eq!(c.execute(), 4); assert_eq!(0x24, c.registers.l); assert_eq!(c.registers.flags.to_byte(), CF);           // INC L
    assert_eq!(c.execute(), 4); assert_eq!(0x23, c.registers.l); assert_eq!(c.registers.flags.to_byte(), NF|CF);        // DEC L
}

#[test]
fn inc_dec_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    c.bus.write_byte(0x1000, 0x00);
    c.bus.write_byte(0x1001, 0x3F);
    c.bus.write_byte(0x1002, 0x7F);
    c.bus.load_bin("bin/inc_dec_i_hl_ix_iy.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute();
    }
    assert_eq!(c.execute(), 11); assert_eq!(0xFF, c.bus.read_byte(0x1000)); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF);  // DEC (HL)
    assert_eq!(c.execute(), 11); assert_eq!(0x00, c.bus.read_byte(0x1000)); assert_eq!(c.registers.flags.to_byte(), ZF|HF);     // INC (HL)
    assert_eq!(c.execute(), 23); assert_eq!(0x40, c.bus.read_byte(0x1001)); assert_eq!(c.registers.flags.to_byte(), HF);        // INC (IX+1)
    assert_eq!(c.execute(), 23); assert_eq!(0x3F, c.bus.read_byte(0x1001)); assert_eq!(c.registers.flags.to_byte(), HF|NF);     // DEC (IX+1)
    assert_eq!(c.execute(), 23); assert_eq!(0x80, c.bus.read_byte(0x1002)); assert_eq!(c.registers.flags.to_byte(), SF|HF|VF);  // INC (IY-1)
    assert_eq!(c.execute(), 23); assert_eq!(0x7F, c.bus.read_byte(0x1002)); assert_eq!(c.registers.flags.to_byte(), HF|PF|NF);  // DEC (IY-1)
}

#[test]
fn inc_dec_ss_ix_iy_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/inc_dec_ss_ix_iy.bin", 0).unwrap();
    for _ in 0..6 {
        c.execute();
    }
    assert_eq!(c.execute(), 6);  assert_eq!(0xFFFF, c.registers.get_bc());      // DEC BC
    assert_eq!(c.execute(), 6);  assert_eq!(0x0000, c.registers.get_bc());      // INC BC
    assert_eq!(c.execute(), 6);  assert_eq!(0x0000, c.registers.get_de());      // INC DE
    assert_eq!(c.execute(), 6);  assert_eq!(0xFFFF, c.registers.get_de());      // DEC DE
    assert_eq!(c.execute(), 6);  assert_eq!(0x0100, c.registers.get_hl());      // INC HL
    assert_eq!(c.execute(), 6);  assert_eq!(0x00FF, c.registers.get_hl());      // DEC HL
    assert_eq!(c.execute(), 6);  assert_eq!(0x1112, c.sp);                      // INC SP
    assert_eq!(c.execute(), 6);  assert_eq!(0x1111, c.sp);                      // DEC SP
    assert_eq!(c.execute(), 10); assert_eq!(0x1000, c.ix);                      // INC IX
    assert_eq!(c.execute(), 10); assert_eq!(0x0FFF, c.ix);                      // DEC IX
    assert_eq!(c.execute(), 10); assert_eq!(0x1235, c.iy);                      // INC IY
    assert_eq!(c.execute(), 10); assert_eq!(0x1234, c.iy);                      // DEC IY
}

#[test]
fn djnz_asm() {
    let mut c = CPU::new();
    c.bus.load_bin("bin/djnz.bin", 0x0204).unwrap();
    c.pc = 0x0204;
    assert_eq!(c.execute(), 7);  assert_eq!(0x03, c.registers.b);
    assert_eq!(c.execute(), 4);  assert_eq!(0x00, c.registers.a);
    assert_eq!(c.execute(), 4);  assert_eq!(0x01, c.registers.a);
    assert_eq!(c.execute(), 13); assert_eq!(0x02, c.registers.b); assert_eq!(0x0207, c.pc);
    assert_eq!(c.execute(), 4);  assert_eq!(0x02, c.registers.a);
    assert_eq!(c.execute(), 13); assert_eq!(0x01, c.registers.b); assert_eq!(0x0207, c.pc);
    assert_eq!(c.execute(), 4);  assert_eq!(0x03, c.registers.a);
    assert_eq!(c.execute(), 8);  assert_eq!(0x00, c.registers.b); assert_eq!(0x020A, c.pc);
}

#[test]
fn ld_b() {
    let mut c = CPU::new();
    c.registers.b = 0x11;
    c.registers.c = 0x15;
    c.registers.d = 0x1F;
    c.registers.e = 0x21;
    c.registers.h = 0x25;
    c.registers.l = 0x2F;
    c.bus.write_byte(0x252f, 0x31);
    c.registers.a = 0x3F;
    c.bus.write_byte(0x0000, 0x40);
    c.bus.write_byte(0x0001, 0x41);
    c.bus.write_byte(0x0002, 0x42);
    c.bus.write_byte(0x0003, 0x43);
    c.bus.write_byte(0x0004, 0x44);
    c.bus.write_byte(0x0005, 0x45);
    c.bus.write_byte(0x0006, 0x46);
    c.bus.write_byte(0x0007, 0x47);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.b, 0x11);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.b, 0x15);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.b, 0x1f);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.b, 0x21);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.b, 0x25);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.b, 0x2f);
    assert_eq!(c.execute(),7);
    assert_eq!(c.registers.b, 0x31);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.b, 0x3f);
    assert_eq!(c.pc, 8);
}

#[test]
fn ld_c() {
    let mut c = CPU::new();
    c.registers.b = 0x11;
    c.registers.c = 0x15;
    c.registers.d = 0x1F;
    c.registers.e = 0x21;
    c.registers.h = 0x25;
    c.registers.l = 0x2F;
    c.bus.write_byte(0x252f, 0x31);
    c.registers.a = 0x3F;
    c.bus.write_byte(0x0000, 0x48);
    c.bus.write_byte(0x0001, 0x49);
    c.bus.write_byte(0x0002, 0x4a);
    c.bus.write_byte(0x0003, 0x4b);
    c.bus.write_byte(0x0004, 0x4c);
    c.bus.write_byte(0x0005, 0x4d);
    c.bus.write_byte(0x0006, 0x4e);
    c.bus.write_byte(0x0007, 0x4f);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.c, 0x11);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.c, 0x11);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.c, 0x1f);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.c, 0x21);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.c, 0x25);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.c, 0x2f);
    assert_eq!(c.execute(),7);
    assert_eq!(c.registers.c, 0x31);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.c, 0x3f);
    assert_eq!(c.pc, 8);
}

#[test]
fn ld_d() {
    let mut c = CPU::new();
    c.registers.b = 0x11;
    c.registers.c = 0x15;
    c.registers.d = 0x1F;
    c.registers.e = 0x21;
    c.registers.h = 0x25;
    c.registers.l = 0x2F;
    c.bus.write_byte(0x252f, 0x31);
    c.registers.a = 0x3F;
    c.bus.write_byte(0x0000, 0x50);
    c.bus.write_byte(0x0001, 0x51);
    c.bus.write_byte(0x0002, 0x52);
    c.bus.write_byte(0x0003, 0x53);
    c.bus.write_byte(0x0004, 0x54);
    c.bus.write_byte(0x0005, 0x55);
    c.bus.write_byte(0x0006, 0x56);
    c.bus.write_byte(0x0007, 0x57);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.d, 0x11);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.d, 0x15);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.d, 0x15);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.d, 0x21);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.d, 0x25);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.d, 0x2f);
    assert_eq!(c.execute(),7);
    assert_eq!(c.registers.d, 0x31);
    assert_eq!(c.execute(),4);
    assert_eq!(c.registers.d, 0x3f);
    assert_eq!(c.pc, 8);
}

#[test]
    fn ld_e() {
        let mut c = CPU::new();
        c.registers.b = 0x11;
        c.registers.c = 0x15;
        c.registers.d = 0x1F;
        c.registers.e = 0x21;
        c.registers.h = 0x25;
        c.registers.l = 0x2F;
        c.bus.write_byte(0x252f, 0x31);
        c.registers.a = 0x3F;
        c.bus.write_byte(0x0000, 0x58);
        c.bus.write_byte(0x0001, 0x59);
        c.bus.write_byte(0x0002, 0x5a);
        c.bus.write_byte(0x0003, 0x5b);
        c.bus.write_byte(0x0004, 0x5c);
        c.bus.write_byte(0x0005, 0x5d);
        c.bus.write_byte(0x0006, 0x5e);
        c.bus.write_byte(0x0007, 0x5f);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.e, 0x11);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.e, 0x15);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.e, 0x1f);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.e, 0x1f);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.e, 0x25);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.e, 0x2f);
        assert_eq!(c.execute(),7);
        assert_eq!(c.registers.e, 0x31);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.e, 0x3f);
        assert_eq!(c.pc, 8);
    }

    #[test]
    fn ld_h() {
        let mut c = CPU::new();
        c.registers.b = 0x11;
        c.registers.c = 0x15;
        c.registers.d = 0x1F;
        c.registers.e = 0x21;
        c.registers.h = 0x25;
        c.registers.l = 0x2F;
        c.bus.write_byte(0x2f2f, 0x31);
        c.registers.a = 0x3F;
        c.bus.write_byte(0x0000, 0x60);
        c.bus.write_byte(0x0001, 0x61);
        c.bus.write_byte(0x0002, 0x62);
        c.bus.write_byte(0x0003, 0x63);
        c.bus.write_byte(0x0004, 0x64);
        c.bus.write_byte(0x0005, 0x65);
        c.bus.write_byte(0x0006, 0x66);
        c.bus.write_byte(0x0007, 0x67);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.h, 0x11);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.h, 0x15);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.h, 0x1f);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.h, 0x21);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.h, 0x21);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.h, 0x2f);
        assert_eq!(c.execute(),7);
        assert_eq!(c.registers.h, 0x31);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.h, 0x3f);
        assert_eq!(c.pc, 8);
    }

    #[test]
    fn ld_l() {
        let mut c = CPU::new();
        c.registers.b = 0x11;
        c.registers.c = 0x15;
        c.registers.d = 0x1F;
        c.registers.e = 0x21;
        c.registers.h = 0x25;
        c.registers.l = 0x2F;
        c.bus.write_byte(0x2525, 0x31);
        c.registers.a = 0x3F;
        c.bus.write_byte(0x0000, 0x68);
        c.bus.write_byte(0x0001, 0x69);
        c.bus.write_byte(0x0002, 0x6a);
        c.bus.write_byte(0x0003, 0x6b);
        c.bus.write_byte(0x0004, 0x6c);
        c.bus.write_byte(0x0005, 0x6d);
        c.bus.write_byte(0x0006, 0x6e);
        c.bus.write_byte(0x0007, 0x6f);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.l, 0x11);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.l, 0x15);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.l, 0x1f);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.l, 0x21);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.l, 0x25);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.l, 0x25);
        assert_eq!(c.execute(),7);
        assert_eq!(c.registers.l, 0x31);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.l, 0x3f);
        assert_eq!(c.pc, 8);
    }

    #[test]
    fn ld_hl_r() {
        let mut c = CPU::new();
        c.registers.b = 0x11;
        c.registers.c = 0x15;
        c.registers.d = 0x1F;
        c.registers.e = 0x21;
        c.registers.h = 0x25;
        c.registers.l = 0x2F;
        c.bus.write_byte(0x2f2f, 0x31);
        c.registers.a = 0x3F;
        c.bus.write_byte(0x0000, 0x70);
        c.bus.write_byte(0x0001, 0x71);
        c.bus.write_byte(0x0002, 0x72);
        c.bus.write_byte(0x0003, 0x73);
        c.bus.write_byte(0x0004, 0x74);
        c.bus.write_byte(0x0005, 0x75);
        c.bus.write_byte(0x0006, 0x77);
        assert_eq!(c.execute(),7);
        assert_eq!(c.bus.read_byte(0x252f), 0x11);
        assert_eq!(c.execute(),7);
        assert_eq!(c.bus.read_byte(0x252f), 0x15);
        assert_eq!(c.execute(),7);
        assert_eq!(c.bus.read_byte(0x252f), 0x1f);
        assert_eq!(c.execute(),7);
        assert_eq!(c.bus.read_byte(0x252f), 0x21);
        assert_eq!(c.execute(),7);
        assert_eq!(c.bus.read_byte(0x252f), 0x25);
        assert_eq!(c.execute(),7);
        assert_eq!(c.bus.read_byte(0x252f), 0x2f);
        assert_eq!(c.execute(),7);
        assert_eq!(c.bus.read_byte(0x252f), 0x3f);
        assert_eq!(c.pc, 7);
    }

    #[test]
    fn ld_a() {
        let mut c = CPU::new();
        c.registers.b = 0x11;
        c.registers.c = 0x15;
        c.registers.d = 0x1F;
        c.registers.e = 0x21;
        c.registers.h = 0x25;
        c.registers.l = 0x2F;
        c.bus.write_byte(0x252f, 0x31);
        c.registers.a = 0x3F;
        c.bus.write_byte(0x0000, 0x78);
        c.bus.write_byte(0x0001, 0x79);
        c.bus.write_byte(0x0002, 0x7a);
        c.bus.write_byte(0x0003, 0x7b);
        c.bus.write_byte(0x0004, 0x7c);
        c.bus.write_byte(0x0005, 0x7d);
        c.bus.write_byte(0x0006, 0x7e);
        c.bus.write_byte(0x0007, 0x7f);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.a, 0x11);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.a, 0x15);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.a, 0x1f);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.a, 0x21);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.a, 0x25);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.a, 0x2f);
        assert_eq!(c.execute(),7);
        assert_eq!(c.registers.a, 0x31);
        assert_eq!(c.execute(),4);
        assert_eq!(c.registers.a, 0x31);
        assert_eq!(c.pc, 8);
    }

    #[test]
    fn hlt() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x76);
        c.execute();
        assert_eq!(c.halt, true);
        assert_eq!(c.pc, 1);
    }

    #[test]
    fn ld_b_ix_d() {
        let mut c = CPU::new();
        c.ix = 0x25AF;
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x46);
        c.bus.write_byte(0x0002, 0x19);
        c.bus.write_byte(0x25C8, 0x39);
        assert_eq!(c.execute(), 19);
        assert_eq!(c.registers.b, 0x39);
        assert_eq!(c.pc, 3);
    }

    #[test]
    fn ld_b_iy_d() {
        let mut c = CPU::new();
        c.iy = 0x25AF;
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0x46);
        c.bus.write_byte(0x0002, 0x19);
        c.bus.write_byte(0x25C8, 0x39);
        assert_eq!(c.execute(), 19);
        assert_eq!(c.registers.b, 0x39);
        assert_eq!(c.pc, 3);
    }

    #[test]
    fn ld_ix_d_c() {
        let mut c = CPU::new();
        c.registers.c = 0x1C;
        c.ix = 0x3100;
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x71);
        c.bus.write_byte(0x0002, 0x06);
        assert_eq!(c.execute(), 19);
        assert_eq!(c.bus.read_byte(0x3106), 0x1C);
        assert_eq!(c.pc, 3);
    }

    #[test]
    fn ld_ix_d_n() {
        let mut c = CPU::new();
        c.ix = 0x219A;
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x36);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x0003, 0x5A);
        assert_eq!(c.execute(), 19);
        assert_eq!(c.bus.read_byte(0x219F), 0x5A);
        assert_eq!(c.pc, 4);
    }

    #[test]
    fn ld_a_bc() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x0a);
        c.bus.write_byte(0x100, 0x65);
        c.registers.set_bc(0x100);
        assert_eq!(c.execute(), 7);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0x65);
    }

    #[test]
    fn ld_a_de() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x1a);
        c.bus.write_byte(0x100, 0x65);
        c.registers.set_de(0x100);
        assert_eq!(c.execute(), 7);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0x65);
    }

    #[test]
    fn ld_nn_a() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x32);
        c.bus.write_byte(0x0001, 0x00);
        c.bus.write_byte(0x0002, 0xff);
        c.registers.a = 0x56;
        assert_eq!(c.execute(), 13);
        assert_eq!(c.pc, 0x0003);
        assert_eq!(c.bus.read_byte(0xff00), 0x56);
    }

    #[test]
    fn ld_a_r() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x5F);
        c.r = 0x56;
        assert_eq!(c.execute(), 9);
        assert_eq!(c.pc, 0x0002);
        assert_eq!(c.registers.a, 0x56);
    }

    #[test]
    fn ld_dd_nn() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x21);
        c.bus.write_byte(0x0001, 0x00);
        c.bus.write_byte(0x0002, 0x50);
        assert_eq!(c.execute(), 10);
        assert_eq!(c.pc, 0x0003);
        assert_eq!(c.registers.get_hl(), 0x5000);
    }

    #[test]
    fn ld_ix_nn() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x21);
        c.bus.write_byte(0x0002, 0xA2);
        c.bus.write_byte(0x0003, 0x45);
        assert_eq!(c.execute(), 14);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.ix, 0x45A2);
    }

    #[test]
    fn ld_hl_nn() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x2A);
        c.bus.write_byte(0x0001, 0x45);
        c.bus.write_byte(0x0002, 0x45);
        c.bus.write_byte(0x4545, 0x37);
        c.bus.write_byte(0x4546, 0xA1);
        assert_eq!(c.execute(), 16);
        assert_eq!(c.pc, 0x0003);
        assert_eq!(c.registers.get_hl(), 0xA137);
    }

    #[test]
    fn ld_bc_cnn() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x4B);
        c.bus.write_byte(0x0002, 0x30);
        c.bus.write_byte(0x0003, 0x21);
        c.bus.write_byte(0x2130, 0x65);
        c.bus.write_byte(0x2131, 0x78);
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.registers.get_bc(), 0x7865);
    }

    #[test]
    fn ld_de_cnn() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x5B);
        c.bus.write_byte(0x0002, 0x30);
        c.bus.write_byte(0x0003, 0x21);
        c.bus.write_byte(0x2130, 0x65);
        c.bus.write_byte(0x2131, 0x78);
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.registers.get_de(), 0x7865);
    }

    #[test]
    fn ld_hl_cnn() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x6B);
        c.bus.write_byte(0x0002, 0x30);
        c.bus.write_byte(0x0003, 0x21);
        c.bus.write_byte(0x2130, 0x65);
        c.bus.write_byte(0x2131, 0x78);
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.registers.get_hl(), 0x7865);
    }

    #[test]
    fn ld_sp_cnn() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x7B);
        c.bus.write_byte(0x0002, 0x30);
        c.bus.write_byte(0x0003, 0x21);
        c.bus.write_byte(0x2130, 0x65);
        c.bus.write_byte(0x2131, 0x78);
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.sp, 0x7865);
    }

    #[test]
    fn ld_ix_cnn() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x2A);
        c.bus.write_byte(0x0002, 0x66);
        c.bus.write_byte(0x0003, 0x66);
        c.bus.write_byte(0x6666, 0x92);
        c.bus.write_byte(0x6667, 0xDA);
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.ix, 0xDA92);
    }

    #[test]
    fn ld_iy_cnn() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0x2A);
        c.bus.write_byte(0x0002, 0x66);
        c.bus.write_byte(0x0003, 0x66);
        c.bus.write_byte(0x6666, 0x92);
        c.bus.write_byte(0x6667, 0xDA);
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.iy, 0xDA92);
    }

    #[test]
    fn ld_cnn_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x22);
        c.bus.write_byte(0x0001, 0x29);
        c.bus.write_byte(0x0002, 0xB2);
        c.registers.set_hl(0x483A);
        assert_eq!(c.execute(), 16);
        assert_eq!(c.pc, 0x0003);
        assert_eq!(c.bus.read_byte(0xB229), 0x3A);
        assert_eq!(c.bus.read_byte(0xB22A), 0x48);
    }

    #[test]
    fn ld_ann_bc() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x43);
        c.bus.write_byte(0x0002, 0x00);
        c.bus.write_byte(0x0003, 0x10);
        c.registers.set_bc(0x4644);
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.bus.read_byte(0x1000), 0x44);
        assert_eq!(c.bus.read_byte(0x1001), 0x46);
    }

    #[test]
    fn ld_ann_de() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x53);
        c.bus.write_byte(0x0002, 0x00);
        c.bus.write_byte(0x0003, 0x10);
        c.registers.set_de(0x4644);
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.bus.read_byte(0x1000), 0x44);
        assert_eq!(c.bus.read_byte(0x1001), 0x46);
    }

    #[test]
    fn ld_ann_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x63);
        c.bus.write_byte(0x0002, 0x00);
        c.bus.write_byte(0x0003, 0x10);
        c.registers.set_hl(0x4644);
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.bus.read_byte(0x1000), 0x44);
        assert_eq!(c.bus.read_byte(0x1001), 0x46);
    }

    #[test]
    fn ld_ann_sp() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x73);
        c.bus.write_byte(0x0002, 0x00);
        c.bus.write_byte(0x0003, 0x10);
        c.sp = 0x4644;
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.bus.read_byte(0x1000), 0x44);
        assert_eq!(c.bus.read_byte(0x1001), 0x46);
    }

    #[test]
    fn ld_ann_ix() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x22);
        c.bus.write_byte(0x0002, 0x38);
        c.bus.write_byte(0x0003, 0x88);
        c.ix = 0x4174;
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.bus.read_byte(0x8838), 0x74);
        assert_eq!(c.bus.read_byte(0x8839), 0x41);
    }

    #[test]
    fn ld_ann_iy() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0x22);
        c.bus.write_byte(0x0002, 0x38);
        c.bus.write_byte(0x0003, 0x88);
        c.iy = 0x4174;
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 0x0004);
        assert_eq!(c.bus.read_byte(0x8838), 0x74);
        assert_eq!(c.bus.read_byte(0x8839), 0x41);
    }

    #[test]
    fn ld_sp_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xF9);
        c.registers.h = 0x50;
        c.registers.l = 0x6c;
        assert_eq!(c.execute(), 6);
        assert_eq!(c.pc, 1);
        assert_eq!(c.sp, 0x506c)
    }

    #[test]
    fn ld_sp_ix() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xF9);
        c.ix = 0x98DA;
        assert_eq!(c.execute(), 10);
        assert_eq!(c.pc, 2);
        assert_eq!(c.sp, 0x98DA)
    }

    #[test]
    fn ld_sp_iy() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xF9);
        c.iy = 0x98DA;
        assert_eq!(c.execute(), 10);
        assert_eq!(c.pc, 2);
        assert_eq!(c.sp, 0x98DA)
    }

    #[test]
    fn push_af() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xF5);
        c.registers.a = 0x22;
        c.registers.flags.from_byte(0x33);
        c.sp = 0x1007;
        assert_eq!(c.registers.flags.to_byte(), 0b00110011);
        assert_eq!(c.execute(), 11);
        assert_eq!(c.pc, 1);
        assert_eq!(c.sp, 0x1005);
        assert_eq!(c.bus.read_byte(0x1005), 0x33);
        assert_eq!(c.bus.read_byte(0x1006), 0x22);
        assert_eq!(c.sp, 0x1005);
    }

    #[test]
    fn push_ix() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xE5);
        c.ix = 0x2233;
        c.sp = 0x1007;
        assert_eq!(c.execute(), 15);
        assert_eq!(c.pc, 2);
        assert_eq!(c.bus.read_byte(0x1005), 0x33);
        assert_eq!(c.bus.read_byte(0x1006), 0x22);
        assert_eq!(c.sp, 0x1005);
    }

    #[test]
    fn push_iy() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xE5);
        c.iy = 0x2233;
        c.sp = 0x1007;
        assert_eq!(c.execute(), 15);
        assert_eq!(c.pc, 2);
        assert_eq!(c.bus.read_byte(0x1005), 0x33);
        assert_eq!(c.bus.read_byte(0x1006), 0x22);
        assert_eq!(c.sp, 0x1005);
    }

    #[test]
    fn pop_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xE1);
        c.bus.write_byte(0x1000, 0x55);
        c.bus.write_byte(0x1001, 0x33);
        c.sp = 0x1000;
        assert_eq!(c.execute(), 10);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.get_hl(), 0x3355);
        assert_eq!(c.sp, 0x1002);
    }

    #[test]
    fn pop_ix() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xE1);
        c.bus.write_byte(0x1000, 0x55);
        c.bus.write_byte(0x1001, 0x33);
        c.sp = 0x1000;
        assert_eq!(c.execute(), 14);
        assert_eq!(c.pc, 2);
        assert_eq!(c.ix, 0x3355);
        assert_eq!(c.sp, 0x1002);
    }

    #[test]
    fn pop_iy() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xE1);
        c.bus.write_byte(0x1000, 0x55);
        c.bus.write_byte(0x1001, 0x33);
        c.sp = 0x1000;
        assert_eq!(c.execute(), 14);
        assert_eq!(c.pc, 2);
        assert_eq!(c.iy, 0x3355);
        assert_eq!(c.sp, 0x1002);
    }

    #[test]
    fn ex_de_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xEB);
        c.registers.set_de(0x2822);
        c.registers.set_hl(0x499A);
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.get_de(), 0x499A);
        assert_eq!(c.registers.get_hl(), 0x2822);
    }

    #[test]
    fn ex_af_afp() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x08);
        c.registers.set_af(0x9900);
        assert_eq!(c.registers.get_af(), 0x9900);
        c.alt_registers.set_af(0x5944);
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.get_af(), 0x5944);
        assert_eq!(c.alt_registers.get_af(), 0x9900);
    }

    #[test]
    fn exx() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xD9);
        c.registers.set_bc(0x445A);
        c.registers.set_de(0x3DA2);
        c.registers.set_hl(0x8859);
        c.alt_registers.set_bc(0x0988);
        c.alt_registers.set_de(0x9300);
        c.alt_registers.set_hl(0x00E7);
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.get_bc(), 0x0988);
        assert_eq!(c.registers.get_de(), 0x9300);
        assert_eq!(c.registers.get_hl(), 0x00E7);
        assert_eq!(c.alt_registers.get_bc(), 0x445A);
        assert_eq!(c.alt_registers.get_de(), 0x3DA2);
        assert_eq!(c.alt_registers.get_hl(), 0x8859);
    }

    #[test]
    fn ex_sp_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xE3);
        c.registers.set_hl(0x7012);
        c.sp = 0x8856;
        c.bus.write_byte(0x8856, 0x11);
        c.bus.write_byte(0x8857, 0x22);
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.get_hl(), 0x2211);
        assert_eq!(c.bus.read_byte(0x8856), 0x12);
        assert_eq!(c.bus.read_byte(0x8857), 0x70);
        assert_eq!(c.sp, 0x8856);
    }

    #[test]
    fn ex_sp_ix() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xE3);
        c.ix = 0x3988;
        c.sp = 0x0100;
        c.bus.write_byte(0x0100, 0x90);
        c.bus.write_byte(0x0101, 0x48);
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 2);
        assert_eq!(c.ix, 0x4890);
        assert_eq!(c.bus.read_byte(0x0100), 0x88);
        assert_eq!(c.bus.read_byte(0x0101), 0x39);
        assert_eq!(c.sp, 0x0100);
    }

    #[test]
    fn ex_sp_iy() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xE3);
        c.ix = 0x3988;
        c.sp = 0x0100;
        c.bus.write_byte(0x0100, 0x90);
        c.bus.write_byte(0x0101, 0x48);
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 2);
        assert_eq!(c.iy, 0x4890);
        assert_eq!(c.bus.read_byte(0x0100), 0x88);
        assert_eq!(c.bus.read_byte(0x0101), 0x39);
        assert_eq!(c.sp, 0x0100);
    }

    #[test]
    fn ldi() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0xA0);
        c.registers.set_hl(0x1111);
        c.registers.set_de(0x2222);
        c.registers.set_bc(0x07);
        c.bus.write_byte(0x1111, 0x88);
        c.bus.write_byte(0x2222, 0x66);
        assert_eq!(c.execute(), 16);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.get_hl(), 0x1112);
        assert_eq!(c.bus.read_byte(0x1111), 0x88);
        assert_eq!(c.registers.get_de(), 0x2223);
        assert_eq!(c.bus.read_byte(0x2222), 0x88);
        assert_eq!(c.registers.get_bc(), 0x06);
    }

    #[test]
    fn ldir() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0xB0);
        c.registers.set_hl(0x1111);
        c.registers.set_de(0x2222);
        c.registers.set_bc(0x0003);
        c.bus.write_byte(0x1111, 0x88);
        c.bus.write_byte(0x2222, 0x66);
        c.bus.write_byte(0x1112, 0x36);
        c.bus.write_byte(0x2223, 0x59);
        c.bus.write_byte(0x1113, 0xA5);
        c.bus.write_byte(0x2224, 0xC5);
        assert_eq!(c.execute(), 21);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.get_hl(), 0x1114);
        assert_eq!(c.bus.read_byte(0x1111), 0x88);
        assert_eq!(c.bus.read_byte(0x1112), 0x36);
        assert_eq!(c.bus.read_byte(0x1113), 0xA5);
        assert_eq!(c.registers.get_de(), 0x2225);
        assert_eq!(c.bus.read_byte(0x2222), 0x88);
        assert_eq!(c.bus.read_byte(0x2223), 0x36);
        assert_eq!(c.bus.read_byte(0x2224), 0xA5);
        assert_eq!(c.registers.get_bc(), 0x00);
    }

    #[test]
    fn ldd() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0xA8);
        c.registers.set_hl(0x1111);
        c.registers.set_de(0x2222);
        c.registers.set_bc(0x07);
        c.bus.write_byte(0x1111, 0x88);
        c.bus.write_byte(0x2222, 0x66);
        assert_eq!(c.execute(), 16);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.get_hl(), 0x1110);
        assert_eq!(c.bus.read_byte(0x1111), 0x88);
        assert_eq!(c.registers.get_de(), 0x2221);
        assert_eq!(c.bus.read_byte(0x2222), 0x88);
        assert_eq!(c.registers.get_bc(), 0x06);
    }

    #[test]
    fn lddr() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0xB8);
        c.registers.set_hl(0x1114);
        c.registers.set_de(0x2225);
        c.registers.set_bc(0x0003);
        c.bus.write_byte(0x1112, 0x88);
        c.bus.write_byte(0x2223, 0x66);
        c.bus.write_byte(0x1113, 0x36);
        c.bus.write_byte(0x2224, 0x59);
        c.bus.write_byte(0x1114, 0xA5);
        c.bus.write_byte(0x2225, 0xC5);
        assert_eq!(c.execute(), 21);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.get_hl(), 0x1111);
        assert_eq!(c.bus.read_byte(0x1112), 0x88);
        assert_eq!(c.bus.read_byte(0x1113), 0x36);
        assert_eq!(c.bus.read_byte(0x1114), 0xA5);
        assert_eq!(c.registers.get_de(), 0x2222);
        assert_eq!(c.bus.read_byte(0x2223), 0x88);
        assert_eq!(c.bus.read_byte(0x2224), 0x36);
        assert_eq!(c.bus.read_byte(0x2225), 0xA5);
        assert_eq!(c.registers.get_bc(), 0x00);
    }

    #[test]
    fn cpi() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0xA1);
        c.registers.a = 0x3B;
        c.registers.set_hl(0x1111);
        c.registers.set_bc(0x01);
        c.bus.write_byte(0x1111, 0x3B);
        assert_eq!(c.execute(), 16);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.get_hl(), 0x1112);
        assert_eq!(c.registers.get_bc(), 0);
        assert_eq!(c.registers.flags.z, true);
        assert_eq!(c.registers.flags.p, false);
    }

    #[test]
    fn cpir() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0xB1);
        c.registers.a = 0xF3;
        c.registers.set_hl(0x1111);
        c.registers.set_bc(0x07);
        c.bus.write_byte(0x1111, 0x52);
        c.bus.write_byte(0x1112, 0x00);
        c.bus.write_byte(0x1113, 0xF3);
        assert_eq!(c.execute(), 21);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.get_hl(), 0x1114);
        assert_eq!(c.registers.get_bc(), 4);
        assert_eq!(c.registers.flags.z, true);
        assert_eq!(c.registers.flags.p, true);
    }

    #[test]
    fn cpd() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0xA9);
        c.registers.a = 0x3B;
        c.registers.set_hl(0x1111);
        c.registers.set_bc(0x01);
        c.bus.write_byte(0x1111, 0x3B);
        assert_eq!(c.execute(), 16);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.get_hl(), 0x1110);
        assert_eq!(c.registers.get_bc(), 0);
        assert_eq!(c.registers.flags.z, true);
        assert_eq!(c.registers.flags.p, false);
    }

    #[test]
    fn cpdr() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0xB9);
        c.registers.a = 0xF3;
        c.registers.set_hl(0x1118);
        c.registers.set_bc(0x07);
        c.bus.write_byte(0x1116, 0xF3);
        c.bus.write_byte(0x1117, 0x00);
        c.bus.write_byte(0x1118, 0x52);
        assert_eq!(c.execute(), 21);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.get_hl(), 0x1115);
        assert_eq!(c.registers.get_bc(), 4);
        assert_eq!(c.registers.flags.z, true);
        assert_eq!(c.registers.flags.p, true);
    }

    #[test]
    fn add_a_r() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x81);
        c.registers.a = 0x44;
        c.registers.c = 0x11;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0x55);
    }

    #[test]
    fn add_a_n() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xC6);
        c.bus.write_byte(0x0001, 0x33);
        c.registers.a = 0x23;
        assert_eq!(c.execute(), 7);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.a, 0x56);
    }

    #[test]
    fn add_a_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x86);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x22);
        c.registers.a = 0x11;
        c.ix = 0x1000;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.a, 0x33);
    }

    #[test]
    fn add_a_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0x86);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x22);
        c.registers.a = 0x11;
        c.iy = 0x1000;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.a, 0x33);
    }

    #[test]
    fn addc_a_r() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x8E);
        c.bus.write_byte(0x6666, 0x10);
        c.registers.a = 0x16;
        c.registers.flags.c = true;
        c.registers.set_hl(0x6666);
        assert_eq!(c.execute(), 7);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0x27);
    }

    #[test]
    fn addc_a_n() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xCE);
        c.bus.write_byte(0x0001, 0x10);
        c.registers.a = 0x16;
        c.registers.flags.c = true;
        assert_eq!(c.execute(), 7);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.a, 0x27);
    }

    #[test]
    fn sub_r() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x92);
        c.registers.a = 0x29;
        c.registers.d = 0x11;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0x18);
    }

    #[test]
    fn sub_a_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x96);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x22);
        c.registers.a = 0x63;
        c.ix = 0x1000;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.a, 0x41);
    }

    #[test]
    fn sub_a_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0x96);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x22);
        c.registers.a = 0x63;
        c.iy = 0x1000;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.a, 0x41);
    }

    #[test]
    fn sbc_a_r() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x9E);
        c.bus.write_byte(0x3433, 0x05);
        c.registers.a = 0x16;
        c.registers.set_hl(0x3433);
        c.registers.flags.c = true;
        assert_eq!(c.execute(), 7);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0x10);
    }

    #[test]
    fn sbc_a_r_ovf() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x9E);
        c.bus.write_byte(0x3433, 0x01);
        c.registers.a = 0x80;
        c.registers.set_hl(0x3433);
        c.registers.flags.c = true;
        assert_eq!(c.execute(), 7);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0x7E);
        assert_eq!(c.registers.flags.p, true);
    }

    #[test]
    fn sbc_a_n() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDE);
        c.bus.write_byte(0x0001, 0x05);
        c.registers.a = 0x16;
        c.registers.flags.c = true;
        assert_eq!(c.execute(), 7);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.a, 0x10);
    }

    #[test]
    fn sbc_a_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x9E);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x22);
        c.registers.a = 0x63;
        c.registers.flags.c = true;
        c.ix = 0x1000;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.a, 0x40);
    }

    #[test]
    fn sbc_a_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0x9E);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x22);
        c.registers.a = 0x63;
        c.registers.flags.c = true;
        c.iy = 0x1000;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.a, 0x40);
    }

    #[test]
    fn and_r() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xA0);
        c.registers.a = 0xC3;
        c.registers.b = 0x7B;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0x43);
    }

    #[test]
    fn and_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xA6);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x7B);
        c.registers.a = 0xC3;
        c.ix = 0x1000;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.a, 0x43);
    }

    #[test]
    fn and_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xA6);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x7B);
        c.registers.a = 0xC3;
        c.iy = 0x1000;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.a, 0x43);
    }

    #[test]
    fn or_r() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xB4);
        c.registers.a = 0x12;
        c.registers.h = 0x48;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0x5A);
    }

    #[test]
    fn or_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xB6);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x48);
        c.registers.a = 0x12;
        c.ix = 0x1000;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.a, 0x5A);
    }

    #[test]
    fn or_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xB6);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x48);
        c.registers.a = 0x12;
        c.iy = 0x1000;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.a, 0x5A);
    }

    #[test]
    fn xor_n() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xEE);
        c.bus.write_byte(0x0001, 0x5D);
        c.registers.a = 0x96;
        assert_eq!(c.execute(), 7);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.a, 0xCB);
    }

    #[test]
    fn xor_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xAE);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x5D);
        c.registers.a = 0x96;
        c.ix = 0x1000;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.a, 0xCB);
    }

    #[test]
    fn xor_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xAE);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x5D);
        c.registers.a = 0x96;
        c.iy = 0x1000;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.a, 0xCB);
    }

    #[test]
    fn cp_r() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xBB);
        c.registers.a = 0x0A;
        c.registers.e = 0x05;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.flags.z, false);
        assert_eq!(c.registers.flags.c, false);
    }

    #[test]
    fn cp_n() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFE);
        c.bus.write_byte(0x0001, 0x05);
        c.registers.a = 0x0A;
        assert_eq!(c.execute(), 7);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.flags.z, false);
        assert_eq!(c.registers.flags.c, false);
    }

    #[test]
    fn cp_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xBE);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x05);
        c.registers.a = 0x0A;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.flags.z, false);
        assert_eq!(c.registers.flags.c, false);
    }

    #[test]
    fn cp_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xBE);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x1005, 0x05);
        c.registers.a = 0x0A;
        assert_eq!(c.execute(), 19);
        assert_eq!(c.pc, 3);
        assert_eq!(c.registers.flags.z, false);
        assert_eq!(c.registers.flags.c, false);
    }

    #[test]
    fn inc_b() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x04);
        c.registers.b = 0xff;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 0x0001);
        assert_eq!(0, c.registers.b);
        assert_eq!(true, c.registers.flags.z);
    }

    #[test]
    fn inc_c() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x0C);
        c.registers.c = 0xff;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 0x0001);
        assert_eq!(0, c.registers.c);
        assert_eq!(true, c.registers.flags.z);
    }

    #[test]
    fn inc_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x14);
        c.registers.d = 0xff;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 0x0001);
        assert_eq!(0, c.registers.d);
        assert_eq!(true, c.registers.flags.z);
    }

    #[test]
    fn inc_e() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x1C);
        c.registers.e = 0xff;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 0x0001);
        assert_eq!(0, c.registers.e);
        assert_eq!(true, c.registers.flags.z);
    }

    #[test]
    fn inc_h() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x24);
        c.registers.h = 0xff;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 0x0001);
        assert_eq!(0, c.registers.h);
        assert_eq!(true, c.registers.flags.z);
    }

    #[test]
    fn inc_l() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x2C);
        c.registers.l = 0xff;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 0x0001);
        assert_eq!(0, c.registers.l);
        assert_eq!(true, c.registers.flags.z);
    }

    #[test]
    fn inc_c_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x34);
        c.bus.write_byte(0x0001, 0x34);
        c.bus.write_byte(0x100, 0xff);
        c.registers.set_hl(0x100);
        assert_eq!(c.execute(), 11);
        assert_eq!(c.pc, 0x0001);
        assert_eq!(0, c.bus.read_byte(0x100));
        assert_eq!(true, c.registers.flags.z);
        c.execute();
        assert_eq!(c.pc, 0x0002);
        assert_eq!(1, c.bus.read_byte(0x100));
        assert_eq!(false, c.registers.flags.z);
    }

    #[test]
    fn inc_a() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x3C);
        c.registers.a = 0x0f;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 0x0001);
        assert_eq!(0x10, c.registers.a);
        assert_eq!(false, c.registers.flags.z);
        assert_eq!(true, c.registers.flags.h);
    }

    #[test]
    fn inc_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x34);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x105, 0xff);
        c.ix = 0x100;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 0x03);
        assert_eq!(0, c.bus.read_byte(0x105));
        assert_eq!(true, c.registers.flags.z);
    }

    #[test]
    fn inc_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0x34);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x105, 0xff);
        c.iy = 0x100;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 0x03);
        assert_eq!(0, c.bus.read_byte(0x105));
        assert_eq!(true, c.registers.flags.z);
    }

    #[test]
    fn dcr_b() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x05);
        c.bus.write_byte(0x0001, 0x05);
        c.registers.b = 0x01;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(0, c.registers.b);
        assert_eq!(true, c.registers.flags.z);
        c.execute();
        assert_eq!(c.pc, 2);
        assert_eq!(0xff, c.registers.b);
        assert_eq!(false, c.registers.flags.z);
    }

    #[test]
    fn dcr_c() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x0d);
        c.bus.write_byte(0x0001, 0x0d);
        c.registers.c = 0x01;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(0, c.registers.c);
        assert_eq!(true, c.registers.flags.z);
        c.execute();
        assert_eq!(c.pc, 2);
        assert_eq!(0xff, c.registers.c);
        assert_eq!(false, c.registers.flags.z);
    }

    #[test]
    fn dcr_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x15);
        c.bus.write_byte(0x0001, 0x15);
        c.registers.d = 0x01;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(0, c.registers.d);
        assert_eq!(true, c.registers.flags.z);
        c.execute();
        assert_eq!(c.pc, 2);
        assert_eq!(0xff, c.registers.d);
        assert_eq!(false, c.registers.flags.z);
    }

    #[test]
    fn dcr_e() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x1d);
        c.bus.write_byte(0x0001, 0x1d);
        c.registers.e = 0x01;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(0, c.registers.e);
        assert_eq!(true, c.registers.flags.z);
        c.execute();
        assert_eq!(c.pc, 2);
        assert_eq!(0xff, c.registers.e);
        assert_eq!(false, c.registers.flags.z);
    }

    #[test]
    fn dcr_h() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x25);
        c.bus.write_byte(0x0001, 0x25);
        c.registers.h = 0x01;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(0, c.registers.h);
        assert_eq!(true, c.registers.flags.z);
        c.execute();
        assert_eq!(c.pc, 2);
        assert_eq!(0xff, c.registers.h);
        assert_eq!(false, c.registers.flags.z);
    }

    #[test]
    fn dcr_l() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x2d);
        c.bus.write_byte(0x0001, 0x2d);
        c.registers.l = 0x01;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(0, c.registers.l);
        assert_eq!(true, c.registers.flags.z);
        c.execute();
        assert_eq!(c.pc, 2);
        assert_eq!(0xff, c.registers.l);
        assert_eq!(false, c.registers.flags.z);
    }

    #[test]
    fn dcr_m() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x35);
        c.bus.write_byte(0x0001, 0x35);
        c.bus.write_byte(0x100, 0x55);
        c.registers.set_hl(0x0100);
        assert_eq!(c.execute(), 11);
        assert_eq!(c.pc, 1);
        assert_eq!(0x54, c.bus.read_byte(0x0100));
        assert_eq!(false, c.registers.flags.z);
        c.execute();
        assert_eq!(c.pc, 2);
        assert_eq!(0x53, c.bus.read_byte(0x0100));
        assert_eq!(false, c.registers.flags.z);
    }

    #[test]
    fn dcr_a() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x3d);
        c.bus.write_byte(0x0001, 0x3d);
        c.registers.a = 0x01;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(0, c.registers.a);
        assert_eq!(true, c.registers.flags.z);
        c.execute();
        assert_eq!(c.pc, 2);
        assert_eq!(0xff, c.registers.a);
        assert_eq!(false, c.registers.flags.z);
    }

    #[test]
    fn dec_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x35);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x105, 0xff);
        c.ix = 0x100;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 0x03);
        assert_eq!(0xFE, c.bus.read_byte(0x105));
        assert_eq!(false, c.registers.flags.z);
    }

    #[test]
    fn dec_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0x35);
        c.bus.write_byte(0x0002, 0x05);
        c.bus.write_byte(0x105, 0xff);
        c.iy = 0x100;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 0x03);
        assert_eq!(0xFE, c.bus.read_byte(0x105));
        assert_eq!(false, c.registers.flags.z);
    }

    #[test]
    fn daa() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x27);
        c.registers.a = 0x9B;
        c.registers.flags.h = false;
        c.registers.flags.c = false;
        c.execute();
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 1);
        assert_eq!(c.registers.flags.h, true);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn neg_doc() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x44);
        c.registers.a = 0b10011000;
        assert_eq!(c.execute(), 8);
        assert_eq!(c.pc, 2);
        assert_eq!(0b01101000, c.registers.a);
    }

    #[test]
    fn neg_asm() {
        let mut c = CPU::new();
        c.bus.load_bin("bin/neg.bin", 0).unwrap();
        assert_eq!(c.execute(), 7); assert_eq!(c.registers.a, 0x01);                                                        // LD A,0x01
        assert_eq!(c.execute(), 8); assert_eq!(c.registers.a, 0xFF); assert_eq!(c.registers.flags.to_byte(), SF|HF|NF|CF);  // NEG
        assert_eq!(c.execute(), 7); assert_eq!(c.registers.a, 0x00); assert_eq!(c.registers.flags.to_byte(), ZF|HF|CF);     // ADD A,0x01
        assert_eq!(c.execute(), 8); assert_eq!(c.registers.a, 0x00); assert_eq!(c.registers.flags.to_byte(), ZF|NF);        // NEG
        assert_eq!(c.execute(), 7); assert_eq!(c.registers.a, 0x80); assert_eq!(c.registers.flags.to_byte(), SF|PF|NF|CF);  // SUB A,0x80
        assert_eq!(c.execute(), 8); assert_eq!(c.registers.a, 0x80); assert_eq!(c.registers.flags.to_byte(), SF|PF|NF|CF);  // NEG
        assert_eq!(c.execute(), 7); assert_eq!(c.registers.a, 0xC0); assert_eq!(c.registers.flags.to_byte(), SF);           // ADD A,0x40
        assert_eq!(c.execute(), 8); assert_eq!(c.registers.a, 0x40); assert_eq!(c.registers.flags.to_byte(), NF|CF);        // NEG
    }

    #[test]
    fn ccf() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x3f);
        c.bus.write_byte(0x0001, 0x3f);
        assert_eq!(c.execute(), 4);
        assert_eq!(true, c.registers.flags.c);
        assert_eq!(c.pc, 0x0001);
        c.execute();
        assert_eq!(false, c.registers.flags.c);
        assert_eq!(c.pc, 0x0002);
    }

    #[test]
    fn scf() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x37);
        c.bus.write_byte(0x0001, 0x37);
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 0x0001);
        assert_eq!(true, c.registers.flags.c);
        c.execute();
        assert_eq!(c.pc, 0x0002);
        assert_eq!(true, c.registers.flags.c);
    }

    #[test]
    fn add_hl_b() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x09);
        c.registers.set_bc(0x339F);
        c.registers.set_hl(0xA17B);
        assert_eq!(c.execute(), 11);
        assert_eq!(c.registers.h, 0xD5);
        assert_eq!(c.registers.l, 0x1A);
        assert_eq!(c.registers.flags.c, false);
        assert_eq!(c.pc, 1);
    }

    #[test]
    fn add_hl_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x19);
        c.registers.set_de(0x339F);
        c.registers.set_hl(0xA17B);
        assert_eq!(c.execute(), 11);
        assert_eq!(c.registers.h, 0xD5);
        assert_eq!(c.registers.l, 0x1A);
        assert_eq!(c.registers.flags.c, false);
        assert_eq!(c.pc, 1);
    }

    #[test]
    fn add_hl_h() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x29);
        c.registers.set_hl(0x339F);
        assert_eq!(c.execute(), 11);
        assert_eq!(c.registers.h, 0x67);
        assert_eq!(c.registers.l, 0x3e);
        assert_eq!(c.registers.flags.c, false);
        assert_eq!(c.pc, 1);
    }

    #[test]
    fn add_hl_sp() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x39);
        c.sp = 0x339F;
        c.registers.set_hl(0xA17B);
        assert_eq!(c.execute(), 11);
        assert_eq!(c.registers.h, 0xD5);
        assert_eq!(c.registers.l, 0x1A);
        assert_eq!(c.registers.flags.c, false);
        assert_eq!(c.pc, 1);
    }

    #[test]
    fn adc_hl_b() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x4A);
        c.registers.set_bc(0x2222);
        c.registers.set_hl(0x5437);
        c.registers.flags.c = true;
        assert_eq!(c.execute(), 15);
        assert_eq!(c.registers.h, 0x76);
        assert_eq!(c.registers.l, 0x5A);
        assert_eq!(c.pc, 2);
    }

    #[test]
    fn adc_hl_d_ovf() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x5A);
        c.registers.set_de(0x7FF0);
        c.registers.set_hl(0x000F);
        c.registers.flags.c = true;
        assert_eq!(c.execute(), 15);
        assert_eq!(c.registers.h, 0x80);
        assert_eq!(c.registers.l, 0x00);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.flags.p, true);
    }

    #[test]
    fn adc_hl_h_ovf() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x6A);
        c.registers.set_hl(0x000F);
        c.registers.flags.c = true;
        assert_eq!(c.execute(), 15);
        assert_eq!(c.registers.h, 0x00);
        assert_eq!(c.registers.l, 0x1F);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.flags.p, false);
    }

    #[test]
    fn adc_hl_sp_ovf() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x7A);
        c.registers.set_hl(0x7FF0);
        c.sp = 0x000F;
        c.registers.flags.c = true;
        assert_eq!(c.execute(), 15);
        assert_eq!(c.registers.h, 0x80);
        assert_eq!(c.registers.l, 0x00);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.flags.p, true);
    }

    #[test]
    fn sbc_hl_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x52);
        c.registers.set_hl(0x9999);
        c.registers.set_de(0x1111);
        c.registers.flags.c = true;
        assert_eq!(c.execute(), 15);
        assert_eq!(c.registers.h, 0x88);
        assert_eq!(c.registers.l, 0x87);
        assert_eq!(c.pc, 2);
    }

    #[test]
    fn add_ix_bc() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x09);
        c.ix = 0x3333;
        c.registers.set_bc(0x5555);
        assert_eq!(c.execute(), 15);
        assert_eq!(c.ix, 0x8888);
        assert_eq!(c.pc, 2);
    }

    #[test]
    fn add_iy_bc() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0x09);
        c.iy = 0x3333;
        c.registers.set_bc(0x5555);
        assert_eq!(c.execute(), 15);
        assert_eq!(c.iy, 0x8888);
        assert_eq!(c.pc, 2);
    }

    #[test]
    fn inc_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x23);
        c.registers.set_hl(0x1000);
        assert_eq!(c.execute(), 6);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.get_hl(), 0x1001);
    }

    #[test]
    fn inc_ix() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x23);
        c.ix = 0x1000;
        assert_eq!(c.execute(), 10);
        assert_eq!(c.pc, 2);
        assert_eq!(c.ix, 0x1001);
    }

    #[test]
    fn inc_iy() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0x23);
        c.iy = 0x1000;
        assert_eq!(c.execute(), 10);
        assert_eq!(c.pc, 2);
        assert_eq!(c.iy, 0x1001);
    }

    #[test]
    fn dec_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x2B);
        c.registers.set_hl(0x1001);
        assert_eq!(c.execute(), 6);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.get_hl(), 0x1000);
    }

    #[test]
    fn dec_ix() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0x2B);
        c.ix = 0x2006;
        assert_eq!(c.execute(), 10);
        assert_eq!(c.pc, 2);
        assert_eq!(c.ix, 0x2005);
    }

    #[test]
    fn dec_iy() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0x2B);
        c.iy = 0x2006;
        assert_eq!(c.execute(), 10);
        assert_eq!(c.pc, 2);
        assert_eq!(c.iy, 0x2005);
    }

    #[test]
    fn rlca() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x07);
        c.registers.a = 0b10001000;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0b00010001);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rla() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x17);
        c.registers.a = 0b01110110;
        c.registers.flags.c = true;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0b11101101);
        assert_eq!(c.registers.flags.c, false);
    }

    #[test]
    fn rrca() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x0F);
        c.registers.a = 0b00010001;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0b10001000);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rra() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0x1F);
        c.registers.a = 0b11100001;
        c.registers.flags.c = false;
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 1);
        assert_eq!(c.registers.a, 0b01110000);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rlc_a() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xCB);
        c.bus.write_byte(0x0001, 0x07);
        c.registers.a = 0b10001000;
        assert_eq!(c.execute(), 8);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.a, 0b00010001);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rlc_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xCB);
        c.bus.write_byte(0x0001, 0x06);
        c.bus.write_byte(0x2828, 0b10001000);
        c.registers.set_hl(0x2828);
        assert_eq!(c.execute(), 15);
        assert_eq!(c.pc, 2);
        assert_eq!(c.bus.read_byte(0x2828), 0b00010001);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rlc_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x02);
        c.bus.write_byte(0x0003, 0x06);
        c.bus.write_byte(0x1002, 0b10001000);
        c.ix = 0x1000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x1002), 0b00010001);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rlc_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x02);
        c.bus.write_byte(0x0003, 0x06);
        c.bus.write_byte(0x1002, 0b10001000);
        c.iy = 0x1000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x1002), 0b00010001);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rl_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xCB);
        c.bus.write_byte(0x0001, 0x12);
        c.registers.d = 0b10001111;
        c.registers.flags.c = false;
        assert_eq!(c.execute(), 8);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.d, 0b00011110);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rl_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x02);
        c.bus.write_byte(0x0003, 0x16);
        c.bus.write_byte(0x1002, 0b10001111);
        c.registers.flags.c = false;
        c.ix = 0x1000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x1002), 0b00011110);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rl_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x02);
        c.bus.write_byte(0x0003, 0x16);
        c.bus.write_byte(0x1002, 0b10001111);
        c.registers.flags.c = false;
        c.iy = 0x1000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x1002), 0b00011110);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn read_le_dword() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x02);
        c.bus.write_byte(0x0003, 0x16);
        assert_eq!(c.bus.read_le_dword(0), 0xFDCB0216);
    }

    #[test]
    fn rrc_a() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xCB);
        c.bus.write_byte(0x0001, 0x0F);
        c.registers.a = 0b00110001;
        assert_eq!(c.execute(), 8);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.a, 0b10011000);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rrc_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x02);
        c.bus.write_byte(0x0003, 0x0E);
        c.bus.write_byte(0x1002, 0b00110001);
        c.registers.flags.c = false;
        c.ix = 0x1000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x1002), 0b10011000);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rrc_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x02);
        c.bus.write_byte(0x0003, 0x0E);
        c.bus.write_byte(0x1002, 0b00110001);
        c.registers.flags.c = false;
        c.iy = 0x1000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x1002), 0b10011000);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rr_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xCB);
        c.bus.write_byte(0x0001, 0x1E);
        c.bus.write_byte(0x4343, 0b11011101);
        c.registers.set_hl(0x4343);
        assert_eq!(c.execute(), 15);
        assert_eq!(c.pc, 2);
        assert_eq!(c.bus.read_byte(0x4343), 0b01101110);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rr_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x02);
        c.bus.write_byte(0x0003, 0x1E);
        c.bus.write_byte(0x1002, 0b11011101);
        c.registers.flags.c = false;
        c.ix = 0x1000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x1002), 0b01101110);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rr_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x02);
        c.bus.write_byte(0x0003, 0x1E);
        c.bus.write_byte(0x1002, 0b11011101);
        c.registers.flags.c = false;
        c.iy = 0x1000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x1002), 0b01101110);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn sla_l() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xCB);
        c.bus.write_byte(0x0001, 0x25);
        c.registers.l = 0b10110001;
        assert_eq!(c.execute(), 8);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.l, 0b01100010);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn sla_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x02);
        c.bus.write_byte(0x0003, 0x26);
        c.bus.write_byte(0x1002, 0b10110001);
        c.registers.flags.c = false;
        c.ix = 0x1000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x1002), 0b01100010);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn sla_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x02);
        c.bus.write_byte(0x0003, 0x26);
        c.bus.write_byte(0x1002, 0b10110001);
        c.registers.flags.c = false;
        c.iy = 0x1000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x1002), 0b01100010);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn sra_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x02);
        c.bus.write_byte(0x0003, 0x2E);
        c.bus.write_byte(0x1002, 0b10111000);
        c.registers.flags.c = false;
        c.ix = 0x1000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x1002), 0b11011100);
        assert_eq!(c.registers.flags.c, false);
    }

    #[test]
    fn srl_b() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xCB);
        c.bus.write_byte(0x0001, 0x38);
        c.registers.b = 0b10001111;
        assert_eq!(c.execute(), 8);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.b, 0b01000111);
        assert_eq!(c.registers.flags.c, true);
    }

    #[test]
    fn rld() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x6F);
        c.bus.write_byte(0x5000, 0b00110001);
        c.registers.set_hl(0x5000);
        c.registers.a = 0b01111010;
        assert_eq!(c.execute(), 18);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.a, 0b01110011);
        assert_eq!(c.bus.read_byte(0x5000), 0b00011010);
    }

    #[test]
    fn rrd() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xED);
        c.bus.write_byte(0x0001, 0x67);
        c.bus.write_byte(0x5000, 0b00100000);
        c.registers.set_hl(0x5000);
        c.registers.a = 0b10000100;
        assert_eq!(c.execute(), 18);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.a, 0b10000000);
        assert_eq!(c.bus.read_byte(0x5000), 0b01000010);
    }

    #[test]
    fn bit_4_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xCB);
        c.bus.write_byte(0x0001, 0x66);
        c.bus.write_byte(0x4444, 0x10);
        c.registers.set_hl(0x4444);
        assert_eq!(c.execute(), 12);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.flags.z, false);
        assert_eq!(c.bus.read_byte(0x4444), 0x10);
    }

    #[test]
    fn bit_6_ix_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x04);
        c.bus.write_byte(0x0003, 0x76);
        c.bus.write_byte(0x2004, 0x40);
        c.ix = 0x2000;
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x2004), 0x40);
        assert_eq!(c.registers.flags.z, false);
    }

    #[test]
    fn bit_6_iy_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x04);
        c.bus.write_byte(0x0003, 0x76);
        c.bus.write_byte(0x2004, 0x40);
        c.iy = 0x2000;
        assert_eq!(c.execute(), 20);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x2004), 0x40);
        assert_eq!(c.registers.flags.z, false);
    }

    #[test]
    fn set_4_a() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xCB);
        c.bus.write_byte(0x0001, 0xE7);
        assert_eq!(c.execute(), 8);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.a, 0x10);
    }

    #[test]
    fn set_4_hl() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xCB);
        c.bus.write_byte(0x0001, 0xE6);
        c.registers.set_hl(0x4444);
        assert_eq!(c.execute(), 15);
        assert_eq!(c.pc, 2);
        assert_eq!(c.bus.read_byte(0x4444), 0x10);
    }

    #[test]
    fn set_0_ix() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x03);
        c.bus.write_byte(0x0003, 0xC6);
        c.ix = 0x2000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x2003), 0x01);
    }

    #[test]
    fn set_0_iy() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x03);
        c.bus.write_byte(0x0003, 0xC6);
        c.iy = 0x2000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x2003), 0x01);
    }

    #[test]
    fn res_6_d() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xCB);
        c.bus.write_byte(0x0001, 0xB2);
        c.registers.d = 0xFF;
        assert_eq!(c.execute(), 8);
        assert_eq!(c.pc, 2);
        assert_eq!(c.registers.d, 0xBF);
    }

    #[test]
    fn reset_0_ix() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xDD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x03);
        c.bus.write_byte(0x0003, 0xB6);
        c.bus.write_byte(0x2003, 0xFF);
        c.ix = 0x2000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x2003), 0xBF);
    }

    #[test]
    fn reset_0_iy() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xFD);
        c.bus.write_byte(0x0001, 0xCB);
        c.bus.write_byte(0x0002, 0x03);
        c.bus.write_byte(0x0003, 0xB6);
        c.bus.write_byte(0x2003, 0xFF);
        c.iy = 0x2000;
        assert_eq!(c.execute(), 23);
        assert_eq!(c.pc, 4);
        assert_eq!(c.bus.read_byte(0x2003), 0xBF);
    }

    #[test]
    fn jp() {
        let mut c = CPU::new();
        c.bus.write_byte(0x0000, 0xC3);
        c.bus.write_byte(0x0001, 0x00);
        c.bus.write_byte(0x0002, 0x3E);
        assert_eq!(c.execute(), 10);
        assert_eq!(c.pc, 0x3e00);
    }

    #[test]
    fn jr() {
        let mut c = CPU::new();
        c.pc = 0x0480;
        c.bus.write_byte(0x0480, 0x18);
        c.bus.write_byte(0x0481, 0x03);
        assert_eq!(c.execute(), 12);
        assert_eq!(c.pc, 0x0485);
    }

    #[test]
    fn jr_neg() {
        let mut c = CPU::new();
        c.pc = 0x0480;
        c.bus.write_byte(0x0480, 0x18);
        c.bus.write_byte(0x0481, 0xFA);
        assert_eq!(c.execute(), 12);
        assert_eq!(c.pc, 0x047C);
    }

    #[test]
    fn jr_c_e() {
        let mut c = CPU::new();
        c.pc = 0x0480;
        c.bus.write_byte(0x0480, 0x38);
        c.bus.write_byte(0x0481, 0xFA);
        c.registers.flags.c = true;
        assert_eq!(c.execute(), 12);
        assert_eq!(c.pc, 0x047C);
    }

    #[test]
    fn jr_nc_e() {
        let mut c = CPU::new();
        c.pc = 0x0480;
        c.bus.write_byte(0x0480, 0x30);
        c.bus.write_byte(0x0481, 0xFA);
        c.registers.flags.c = false;
        assert_eq!(c.execute(), 12);
        assert_eq!(c.pc, 0x047C);
    }

    #[test]
    fn jr_z_e() {
        let mut c = CPU::new();
        c.pc = 0x0300;
        c.bus.write_byte(0x0300, 0x28);
        c.bus.write_byte(0x0301, 0x03);
        c.registers.flags.z = true;
        assert_eq!(c.execute(), 12);
        assert_eq!(c.pc, 0x0305);
    }

    #[test]
    fn jr_nz_e() {
        let mut c = CPU::new();
        c.pc = 0x0480;
        c.bus.write_byte(0x0480, 0x20);
        c.bus.write_byte(0x0481, 0xFA);
        c.registers.flags.z = false;
        assert_eq!(c.execute(), 12);
        assert_eq!(c.pc, 0x047C);
    }

    #[test]
    fn jp_hl() {
        let mut c = CPU::new();
        c.pc = 0x1000;
        c.bus.write_byte(0x1000, 0xE9);
        c.registers.set_hl(0x4800);
        assert_eq!(c.execute(), 4);
        assert_eq!(c.pc, 0x4800);
    }

    #[test]
    fn jp_ix() {
        let mut c = CPU::new();
        c.pc = 0x1000;
        c.bus.write_byte(0x1000, 0xDD);
        c.bus.write_byte(0x1001, 0xE9);
        c.ix = 0x4800;
        assert_eq!(c.execute(), 8);
        assert_eq!(c.pc, 0x4800);
    }

    #[test]
    fn jp_iy() {
        let mut c = CPU::new();
        c.pc = 0x1000;
        c.bus.write_byte(0x1000, 0xFD);
        c.bus.write_byte(0x1001, 0xE9);
        c.iy = 0x4800;
        assert_eq!(c.execute(), 8);
        assert_eq!(c.pc, 0x4800);
    }

    #[test]
    fn call_nn() {
        let mut c = CPU::new();
        c.pc = 0x1A47;
        c.sp = 0x3002;
        c.bus.write_byte(0x1A47, 0xCD);
        c.bus.write_byte(0x1A48, 0x35);
        c.bus.write_byte(0x1A49, 0x21);
        assert_eq!(c.execute(), 17);
        assert_eq!(c.bus.read_byte(0x3001), 0x1A);
        assert_eq!(c.bus.read_byte(0x3000), 0x4A);
        assert_eq!(c.sp, 0x3000);
        assert_eq!(c.pc, 0x2135);
    }

    #[test]
    fn call_cc_nn() {
        let mut c = CPU::new();
        c.registers.flags.c = false;
        c.pc = 0x1A47;
        c.sp = 0x3002;
        c.bus.write_byte(0x1A47, 0xD4);
        c.bus.write_byte(0x1A48, 0x35);
        c.bus.write_byte(0x1A49, 0x21);
        assert_eq!(c.execute(), 17);
        assert_eq!(c.bus.read_byte(0x3001), 0x1A);
        assert_eq!(c.bus.read_byte(0x3000), 0x4A);
        assert_eq!(c.sp, 0x3000);
        assert_eq!(c.pc, 0x2135);
    }

    #[test]
    fn ret() {
        let mut c = CPU::new();
        c.pc = 0x3535;
        c.sp = 0x2000;
        c.bus.write_byte(0x3535, 0xC9);
        c.bus.write_byte(0x2000, 0xB5);
        c.bus.write_byte(0x2001, 0x18);
        assert_eq!(c.execute(), 10);
        assert_eq!(c.sp, 0x2002);
        assert_eq!(c.pc, 0x18B5);
    }

    #[test]
    fn ret_cc() {
        let mut c = CPU::new();
        c.registers.flags.s = true;
        c.pc = 0x3535;
        c.sp = 0x2000;
        c.bus.write_byte(0x3535, 0xF8);
        c.bus.write_byte(0x2000, 0xB5);
        c.bus.write_byte(0x2001, 0x18);
        assert_eq!(c.execute(), 11);
        assert_eq!(c.sp, 0x2002);
        assert_eq!(c.pc, 0x18B5);
    }

    #[test]
    fn rst() {
        let mut c = CPU::new();
        c.pc = 0x15B3;
        c.bus.write_byte(0x15B3, 0xDF);
        assert_eq!(c.execute(), 11);
        assert_eq!(c.pc, 0x0018);
    }