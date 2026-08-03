use crate::{bus::Bus, bus::FlatBus, cpu::CPU};

// carry flag
const CF: u8 = 1 << 0;

// add/subtract flag
const NF: u8 = 1 << 1;

// overflow flag (same as parity)
const VF: u8 = 1 << 2;

// parity flag (same as overflow)
const PF: u8 = 1 << 2;

// undocumented 'X' flag
// const XF: u8 = 1 << 3;

// half carry flag
const HF: u8 = 1 << 4;

// undocumented 'Y' flag
// const YF: u8 = 1 << 5;

// zero flag
const ZF: u8 = 1 << 6;

// sign flag
const SF: u8 = 1 << 7;

/// Exécute une instruction à répétition (LDIR, CPIR, OTIR...) jusqu'à son
/// terme, en la rejouant tant que le PC n'a pas dépassé l'opcode.
///
/// C'est ainsi qu'une machine réelle la voit : le Z80 n'exécute qu'une
/// itération par instruction et recule le PC pour la rejouer, ce qui laisse
/// une interruption s'intercaler entre deux itérations. Renvoie le total des
/// cycles consommés.
fn run_block(c: &mut CPU, b: &mut FlatBus) -> u32 {
    let start = c.reg.pc;
    let mut total = 0;
    loop {
        total += c.execute(b);
        if c.reg.pc != start {
            return total;
        }
    }
}

/// Durées de référence du Z80 (T-states), d'après les tables Zilog.
///
/// Ce test est un garde-fou : c'est lui qui a mis au jour que toute la plage
/// IN/OUT préfixée ED était comptée pour zéro. Une durée fausse ne casse rien
/// visiblement, elle décale seulement le temps émulé — et sur une machine où
/// la vidéo, les interruptions et le son sont cadencés par ce temps, cela
/// s'entend avant que cela ne se voie.
const REFERENCE_TIMINGS: &[(&str, &[u8], u32)] = &[
    ("NOP", &[0x00], 4),
    ("LD A,B", &[0x78], 4),
    ("LD A,(HL)", &[0x7E], 7),
    ("LD (HL),A", &[0x77], 7),
    ("INC HL", &[0x23], 6),
    ("LD HL,nn", &[0x21, 0x34, 0x12], 10),
    ("PUSH BC", &[0xC5], 11),
    ("POP BC", &[0xC1], 10),
    ("CALL nn", &[0xCD, 0x00, 0x90], 17),
    ("RET", &[0xC9], 10),
    ("JR d", &[0x18, 0x00], 12),
    ("EX AF,AF'", &[0x08], 4),
    ("EXX", &[0xD9], 4),
    ("IN A,(n)", &[0xDB, 0x00], 11),
    ("OUT (n),A", &[0xD3, 0x00], 11),
    ("IN A,(C)", &[0xED, 0x78], 12),
    ("IN B,(C)", &[0xED, 0x40], 12),
    ("OUT (C),A", &[0xED, 0x79], 12),
    ("OUT (C),C", &[0xED, 0x49], 12),
    ("SBC HL,BC", &[0xED, 0x42], 15),
    ("LD (nn),BC", &[0xED, 0x43, 0x00, 0x90], 20),
    ("NEG", &[0xED, 0x44], 8),
    ("IM 1", &[0xED, 0x56], 8),
    ("LD A,R", &[0xED, 0x5F], 9),
    ("RRD", &[0xED, 0x67], 18),
    ("INC IX", &[0xDD, 0x23], 10),
    ("LD L,(IX+d)", &[0xDD, 0x6E, 0x02], 19),
    ("LD (IX+d),A", &[0xDD, 0x77, 0x02], 19),
    ("ADD IX,DE", &[0xDD, 0x19], 15),
    ("PUSH IX", &[0xDD, 0xE5], 15),
    ("BIT 0,(IX+d)", &[0xDD, 0xCB, 0x00, 0x46], 20),
    ("RES 0,B", &[0xCB, 0x80], 8),
    ("BIT 0,(HL)", &[0xCB, 0x46], 12),
    ("LDI", &[0xED, 0xA0], 16),
    ("INI", &[0xED, 0xA2], 16),
    ("OUTI", &[0xED, 0xA3], 16),
];

#[test]
fn instruction_timings_match_the_zilog_tables() {
    for (name, bytes, expected) in REFERENCE_TIMINGS {
        let mut c = CPU::new();
        let mut b = FlatBus::new(0xFFFF);
        for (i, byte) in bytes.iter().enumerate() {
            b.write_byte(i as u16, *byte);
        }
        assert_eq!(c.execute(&mut b), *expected, "duree de {name}");
    }
}

/// Les instructions à répétition ne doivent pas s'exécuter d'un seul tenant :
/// le Z80 en fait une itération, recule le PC et laisse ainsi une interruption
/// s'intercaler. Un LDIR déroulé d'un bloc gèlerait la machine hôte pendant
/// des dizaines de milliers de cycles.
#[test]
fn repeating_instructions_are_interruptible() {
    // Les deux familles ne comptent pas sur le même registre : LDIR et
    // consorts décomptent BC, les transferts par blocs d'I/O décomptent B.
    const COUNT: u16 = 4;
    let cases: &[(&[u8], bool)] = &[
        (&[0xED, 0xB0], false), // LDIR
        (&[0xED, 0xB8], false), // LDDR
        (&[0xED, 0xB1], false), // CPIR (aucune correspondance)
        (&[0xED, 0xB9], false), // CPDR
        (&[0xED, 0xB2], true),  // INIR
        (&[0xED, 0xBA], true),  // INDR
        (&[0xED, 0xB3], true),  // OTIR
        (&[0xED, 0xBB], true),  // OTDR
    ];

    for (opcode, counts_on_b) in cases {
        let count = &COUNT;
        let mut c = CPU::new();
        let mut b = FlatBus::new(0xFFFF);
        b.write_byte(0x0000, opcode[0]);
        b.write_byte(0x0001, opcode[1]);
        c.reg.set_hl(0x4000);
        c.reg.set_de(0x5000);
        if *counts_on_b {
            c.reg.b = *count as u8;
            c.reg.c = 0x00;
        } else {
            c.reg.set_bc(*count);
        }
        c.reg.a = 0xFF; // aucune correspondance possible pour CPIR/CPDR

        let mut iterations = 0;
        let mut cycles = 0;
        while c.reg.pc == 0x0000 {
            cycles += c.execute(&mut b);
            iterations += 1;
            assert!(iterations <= 8, "{opcode:02X?} ne se termine pas");
        }

        assert_eq!(
            iterations, *count as u32,
            "{opcode:02X?} doit rendre la main a chaque iteration"
        );
        assert_eq!(c.reg.pc, 0x0002, "{opcode:02X?} doit finir par avancer");
        assert_eq!(
            cycles,
            21 * (*count as u32 - 1) + 16,
            "{opcode:02X?} : 21 cycles par iteration, 16 pour la derniere"
        );
    }
}

#[test]
fn ld_r_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ld_r_r.bin", 0).unwrap();
    c.reg.a = 0x12;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.b, 0x12); // LD B,A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.c, 0x12); // LD C,A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.d, 0x12); // LD D,A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.e, 0x12); // LD E,A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.h, 0x12); // LD H,A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.l, 0x12); // LD L,A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.a, 0x12); // LD A,A
    c.reg.b = 0x13;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.c, 0x13); // LD C,B
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.d, 0x13); // LD D,C
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.e, 0x13); // LD E,D
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.h, 0x13); // LD H,E
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.l, 0x13); // LD L,H
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.a, 0x13); // LD A,L
}

#[test]
fn ld_hl_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ld_hl.bin", 0x0100).unwrap();
    c.reg.a = 0x33;
    c.reg.set_hl(0x1000);
    c.reg.pc = 0x0100;
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(b.read_byte(0x1000), 0x33); // LD (HL),A
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.b, 0x33); // LD B,(HL)
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.c, 0x33); // LD C,(HL)
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.d, 0x33); // LD D,(HL)
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.e, 0x33); // LD E,(HL)
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.h, 0x33); // LD H,(HL)
}

#[test]
fn ld_hl_n_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ld_hl_n.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(c.reg.get_hl(), 0x2000); // LD HL,0x2000
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(b.read_byte(0x2000), 0x33); // LD (HL),0x33
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(c.reg.get_hl(), 0x1000); // LD HL,0x1000
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(b.read_byte(0x1000), 0x65); // LD (HL),0x65
}

#[test]
fn ld_ix_iy_n_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ld_ix_iy_n.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(c.reg.get_ix(), 0x2000); // LD IX,0x2000
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x33, b.read_byte(0x2002)); // LD (IX+2),0x33
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x11, b.read_byte(0x1FFE)); // LD (IX-2),0x11
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1000, c.reg.get_iy()); // LD IY,0x1000
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x22, b.read_byte(0x1001)); // LD (IY+1),0x22
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x44, b.read_byte(0x0FFF)); // LD (IY-1),0x44
}

#[test]
fn ld_hl_dd_ix_iy_inn_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x02);
    b.write_byte(0x1002, 0x03);
    b.write_byte(0x1003, 0x04);
    b.write_byte(0x1004, 0x05);
    b.write_byte(0x1005, 0x06);
    b.write_byte(0x1006, 0x07);
    b.write_byte(0x1007, 0x08);
    b.load_bin("bin/ld_hl_dd_ix_iy_inn.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x0201, c.reg.get_hl()); // LD HL,(0x1000)
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(0x0302, c.reg.get_bc()); // LD BC,(0x1001)
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(0x0403, c.reg.get_de()); // LD DE,(0x1002)
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x0504, c.reg.get_hl()); // LD HL,(0x1003)
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(0x0605, c.reg.sp); // LD SP,(0x1004)
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(0x0706, c.reg.get_ix(),); // LD IX,(0x1004)
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(0x0807, c.reg.get_iy()); // LD IY,(0x1005)
}

#[test]
fn ld_ix_iy_nn_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ld_ix_iy_nn.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1234, c.reg.get_bc()); // LD BC,0x1234
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x5678, c.reg.get_de()); // LD DE,0x5678
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x9ABC, c.reg.get_hl()); // LD HL,0x9ABC
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1368, c.reg.sp); // LD SP,0x1368
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x4321, c.reg.get_ix(),); // LD IX,0x4321
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x8765, c.reg.get_iy()); // LD IY,0x8765
}

#[test]
fn ld_sp_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ld_sp_hl_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1234, c.reg.get_hl()); // LD HL,0x1234
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x5678, c.reg.get_ix(),); // LD IX,0x5678
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x9ABC, c.reg.get_iy()); // LD IY,0x9ABC
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x1234, c.reg.sp); // LD SP,HL
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x5678, c.reg.sp); // LD SP,IX
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x9ABC, c.reg.sp); // LD SP,IY
}

#[test]
fn ld_r_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x02);
    b.write_byte(0x1002, 0x03);
    b.write_byte(0x1003, 0x04);
    b.write_byte(0x1004, 0x05);
    b.write_byte(0x1005, 0x06);
    b.write_byte(0x1006, 0x07);
    b.write_byte(0x1007, 0x08);
    b.load_bin("bin/ld_r_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1003, c.reg.get_ix(),); // LD  IX,0x1003
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(4, c.reg.a); // LD  A,(IX+0)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(5, c.reg.b); // LD  B,(IX+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(6, c.reg.c); // LD  C,(IX+2)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(3, c.reg.d); // LD  D,(IX-1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(2, c.reg.e); // LD  E,(IX-2)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(7, c.reg.h); // LD  H,(IX+3)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(1, c.reg.l); // LD  L,(IX-3)
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1004, c.reg.get_iy()); // LD  IY,0x1004
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(5, c.reg.a); // LD  A,(IY+0)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(6, c.reg.b); // LD  B,(IY+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(7, c.reg.c); // LD  C,(IY+2)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(4, c.reg.d); // LD  D,(IY-1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(3, c.reg.e); // LD  E,(IY-2)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(8, c.reg.h); // LD  H,(IY+3)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(2, c.reg.l); // LD  L,(IY-3)
}

#[test]
fn ld_ix_iy_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ld_ix_iy_r.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1003, c.reg.get_ix(),);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x12, c.reg.a);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x12, b.read_byte(0x1003));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x13, c.reg.b);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x13, b.read_byte(0x1004));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x14, c.reg.c);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x14, b.read_byte(0x1005));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x15, c.reg.d);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x15, b.read_byte(0x1002));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x16, c.reg.e);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x16, b.read_byte(0x1001));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x17, c.reg.h);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x17, b.read_byte(0x1006));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x18, c.reg.l);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x18, b.read_byte(0x1000));
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1003, c.reg.get_iy());
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x12, c.reg.a);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x12, b.read_byte(0x1003));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x13, c.reg.b);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x13, b.read_byte(0x1004));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x14, c.reg.c);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x14, b.read_byte(0x1005));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x15, c.reg.d);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x15, b.read_byte(0x1002));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x16, c.reg.e);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x16, b.read_byte(0x1001));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x17, c.reg.h);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x17, b.read_byte(0x1006));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x18, c.reg.l);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x18, b.read_byte(0x1000));
}

#[test]
fn push_pop_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/push_pop.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1234, c.reg.get_bc()); // LD BC,0x1234
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x5678, c.reg.get_de()); // LD DE,0x5678
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x9ABC, c.reg.get_hl()); // LD HL,0x9ABC
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xEF00, c.reg.get_af()); // LD A,0xEF
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x2345, c.reg.get_ix(),); // LD IX,0x2345
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x6789, c.reg.get_iy()); // LD IY,0x6789
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0100, c.reg.sp); // LD SP,0x0100
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0xEF00, b.read_word(0x00FE));
    assert_eq!(0x00FE, c.reg.sp); // PUSH AF
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x1234, b.read_word(0x00FC));
    assert_eq!(0x00FC, c.reg.sp); // PUSH BC
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x5678, b.read_word(0x00FA));
    assert_eq!(0x00FA, c.reg.sp); // PUSH DE
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x9ABC, b.read_word(0x00F8));
    assert_eq!(0x00F8, c.reg.sp); // PUSH HL
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x2345, b.read_word(0x00F6));
    assert_eq!(0x00F6, c.reg.sp); // PUSH IX
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x6789, b.read_word(0x00F4));
    assert_eq!(0x00F4, c.reg.sp); // PUSH IY
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x6789, c.reg.get_af());
    assert_eq!(0x00F6, c.reg.sp); // POP AF
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x2345, c.reg.get_bc());
    assert_eq!(0x00F8, c.reg.sp); // POP BC
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x9ABC, c.reg.get_de());
    assert_eq!(0x00FA, c.reg.sp); // POP DE
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x5678, c.reg.get_hl());
    assert_eq!(0x00FC, c.reg.sp); // POP HL
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1234, c.reg.get_ix(),);
    assert_eq!(0x00FE, c.reg.sp); // POP IX
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0xEF00, c.reg.get_iy());
    assert_eq!(0x0100, c.reg.sp); // POP IY
}

#[test]
fn add_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/add_r.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x0F, c.reg.a);
    assert_eq!(c.flags(), 0); // LD A,0x0F
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x1E, c.reg.a);
    assert_eq!(c.flags(), HF); // ADD A,A
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xE0, c.reg.b); // LD B,0xE0
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xFE, c.reg.a);
    assert_eq!(c.flags(), SF); // ADD A,B
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x81, c.reg.a); // LD A,0x81
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x80, c.reg.c); // LD C,0x80
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.flags(), VF | CF); // ADD A,C
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFF, c.reg.d); // LD D,0xFF
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | HF | CF); // ADD A,D
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x40, c.reg.e); // LD E,0x40
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x40, c.reg.a);
    assert_eq!(c.flags(), 0); // ADD A,E
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x80, c.reg.h); // LD H,0x80
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xC0, c.reg.a);
    assert_eq!(c.flags(), SF); // ADD A,H
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x33, c.reg.l); // LD L,0x33
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xF3, c.reg.a);
    assert_eq!(c.flags(), SF); // ADD A,L
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x37, c.reg.a);
    assert_eq!(c.flags(), CF); // ADD A,0x44
}

#[test]
fn add_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x41);
    b.write_byte(0x1001, 0x61);
    b.write_byte(0x1002, 0x81);
    b.load_bin("bin/add_i_hl_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1000, c.reg.get_hl()); // LD HL,0x1000
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1000, c.reg.get_ix(),); // LD IX,0x1000
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1003, c.reg.get_iy()); // LD IY,0x1003
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, c.reg.a); // LD A,0x00
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x41, c.reg.a);
    assert_eq!(c.flags(), 0); // ADD A,(HL)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xA2, c.reg.a);
    assert_eq!(c.flags(), SF | VF); // ADD A,(IX+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x23, c.reg.a);
    assert_eq!(c.flags(), VF | CF); // ADD A,(IY-1)
}

#[test]
fn add_ixh_ixl_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/add_a_ixh_ixl.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x0F, c.reg.a);
    assert_eq!(c.flags(), 0); // LD A,0x0F
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x1E, c.reg.a);
    assert_eq!(c.flags(), HF); // ADD A,A
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0xE080, c.reg.get_ix(),); // LD  IX,0xE080
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFE, c.reg.a);
    assert_eq!(c.flags(), SF); // ADD A,IXH
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x81, c.reg.a); // LD  A,0x81
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.flags(), VF | CF); // ADD A,IXL
}

#[test]
fn add_a_iyh_iyl_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/add_a_iyh_iyl.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x0F, c.reg.a);
    assert_eq!(c.flags(), 0); // LD A,0x0F
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x1E, c.reg.a);
    assert_eq!(c.flags(), HF); // ADD A,A
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0xE080, c.reg.get_iy()); // LD  IY,0xE080
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFE, c.reg.a);
    assert_eq!(c.flags(), SF); // ADD A,IYH
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x81, c.reg.a); // LD  A,0x81
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.flags(), VF | CF); // ADD A,IYL
}

#[test]
fn adc_a_ixh_ixl_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/adc_a_ixh_ixl.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, c.reg.a); // LD A,0x00
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x4161, c.reg.get_ix(),); // LD IX,0x4161
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF); // ADC A,A
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x41, c.reg.a);
    assert_eq!(c.flags(), 0); // ADC A,IXH
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xA2, c.reg.a);
    assert_eq!(c.flags(), SF | VF); // ADC A,IXL
}

#[test]
fn adc_a_iyh_iyl_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/adc_a_iyh_iyl.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, c.reg.a); // LD A,0x00
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x4161, c.reg.get_iy()); // LD IY,0x4161
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF); // ADC A,A
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x41, c.reg.a);
    assert_eq!(c.flags(), 0); // ADC A,IYH
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xA2, c.reg.a);
    assert_eq!(c.flags(), SF | VF); // ADC A,IYL
}

#[test]
fn adc_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/adc_r.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, c.reg.a); // LD A,0x00
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x41, c.reg.b); // LD B,0x41
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x61, c.reg.c); // LD C,0x61
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x81, c.reg.d); // LD D,0x81
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x41, c.reg.e); // LD E,0x41
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x61, c.reg.h); // LD H,0x61
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x81, c.reg.l); // LD L,0x81
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF); // ADC A,A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x41, c.reg.a);
    assert_eq!(c.flags(), 0); // ADC A,B
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xA2, c.reg.a);
    assert_eq!(c.flags(), SF | VF); // ADC A,C
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x23, c.reg.a);
    assert_eq!(c.flags(), VF | CF); // ADC A,D
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x65, c.reg.a);
    assert_eq!(c.flags(), 0); // ADC A,E
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xC6, c.reg.a);
    assert_eq!(c.flags(), SF | VF); // ADC A,H
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x47, c.reg.a);
    assert_eq!(c.flags(), VF | CF); // ADC A,L
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x49, c.reg.a);
    assert_eq!(c.flags(), 0); // ADC A,0x01
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x0F, c.reg.a); // LD A,0x0F
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x01, c.reg.b); // LD B,0x01
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x10, c.reg.a);
    assert_eq!(c.flags(), HF); // ADC A,B
}

#[test]
fn adc_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x41);
    b.write_byte(0x1001, 0x61);
    b.write_byte(0x1002, 0x81);
    b.write_byte(0x1003, 0x02);
    b.load_bin("bin/adc_i_hl_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1000, c.reg.get_hl()); // LD HL,0x1000
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1000, c.reg.get_ix(),); // LD IX,0x1000
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1003, c.reg.get_iy()); // LD IY,0x1003
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, c.reg.a); // LD A,0x00
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x41, c.reg.a);
    assert_eq!(c.flags(), 0); // ADD A,(HL)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xA2, c.reg.a);
    assert_eq!(c.flags(), SF | VF); // ADC A,(IX+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x23, c.reg.a);
    assert_eq!(c.flags(), VF | CF); // ADC A,(IY-1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x26, c.reg.a);
    assert_eq!(c.flags(), 0); // ADC A,(IX+3)
}

#[test]
fn sub_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/sub_r.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x04, c.reg.a); // LD A,0x04
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x01, c.reg.b); // LD B,0x01
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xF8, c.reg.c); // LD C,0xF8
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x0F, c.reg.d); // LD D,0x0F
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x79, c.reg.e); // LD E,0x79
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xC0, c.reg.h); // LD H,0xC0
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xBF, c.reg.l); // LD L,0xBF
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // SUB A,A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // SUB A,B
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x07, c.reg.a);
    assert_eq!(c.flags(), NF); // SUB A,C
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xF8, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // SUB A,D
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x7F, c.reg.a);
    assert_eq!(c.flags(), HF | VF | NF); // SUB A,E
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xBF, c.reg.a);
    assert_eq!(c.flags(), SF | VF | NF | CF); // SUB A,H
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // SUB A,L
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // SUB A,0x01
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.flags(), NF); // SUB A,0xFE
}

#[test]
fn sub_ixh_ixl_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/sub_ixh_ixl.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x04, c.reg.a); // LD A,0x04
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x01F8, c.reg.get_ix(),); // LD B,0x01
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // SUB A,A
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // SUB A,IXH
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x07, c.reg.a);
    assert_eq!(c.flags(), NF); // SUB A,IXL
}

#[test]
fn sub_iyh_iyl_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/sub_iyh_iyl.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x04, c.reg.a); // LD A,0x04
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x01F8, c.reg.get_iy()); // LD B,0x01
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // SUB A,A
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // SUB A,IXH
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x07, c.reg.a);
    assert_eq!(c.flags(), NF); // SUB A,IXL
}

#[test]
fn cp_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/cp_r.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x04, c.reg.a); // LD A,0x04
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x05, c.reg.b); // LD B,0x05
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x03, c.reg.c); // LD C,0x03
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xff, c.reg.d); // LD D,0xff
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xaa, c.reg.e); // LD E,0xaa
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x80, c.reg.h); // LD H,0x80
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x7f, c.reg.l); // LD L,0x7f
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x04, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // CP A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x04, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // CP B
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x04, c.reg.a);
    assert_eq!(c.flags(), NF); // CP C
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x04, c.reg.a);
    assert_eq!(c.flags(), HF | NF | CF); // CP D
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x04, c.reg.a);
    assert_eq!(c.flags(), HF | NF | CF); // CP E
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x04, c.reg.a);
    assert_eq!(c.flags(), SF | VF | NF | CF); // CP H
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x04, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // CP L
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x04, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // CP 0x04
}

#[test]
fn sub_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x41);
    b.write_byte(0x1001, 0x61);
    b.write_byte(0x1002, 0x81);
    b.load_bin("bin/sub_i_hl_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1000, c.reg.get_hl()); // LD HL,0x1000
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1000, c.reg.get_ix(),); // LD IX,0x1000
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1003, c.reg.get_iy()); // LD IY,0x1003
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, c.reg.a); // LD A,0x00
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xBF, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // SUB A,(HL)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x5E, c.reg.a);
    assert_eq!(c.flags(), VF | NF); // SUB A,(IX+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xFD, c.reg.a);
    assert_eq!(c.flags(), SF | NF | CF); // SUB A,(IY-2)
}

#[test]
fn cp_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x41);
    b.write_byte(0x1001, 0x61);
    b.write_byte(0x1002, 0x22);
    b.load_bin("bin/cp_i_hl_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1000, c.reg.get_hl()); // LD HL,0x1000
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1000, c.reg.get_ix(),); // LD IX,0x1000
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1003, c.reg.get_iy()); // LD IY,0x1003
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x41, c.reg.a); // LD A,0x41
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x41, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // CP (HL)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x41, c.reg.a);
    assert_eq!(c.flags(), SF | NF | CF); // CP (IX+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x41, c.reg.a);
    assert_eq!(c.flags(), HF | NF); // CP (IY-1)
}

#[test]
fn sbc_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/sbc_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute(&mut b);
    }
    // LD  A,0x04
    // LD  B,0x01
    // LD  C,0xF8
    // LD  D,0x0F
    // LD  E,0x79
    // LD  H,0xC0
    // LD  L,0xBF
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // SUB A,A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // SBC A,B (0x00 - 0x01)
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x06, c.reg.a);
    assert_eq!(c.flags(), NF); // SBC A,C (0xFF - 0xF8 - carry)
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xF7, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // SBC A,D (0x06 - 0x0F)
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x7D, c.reg.a);
    assert_eq!(c.flags(), HF | VF | NF); // SBC A,E (0xF7 - 0x79)
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xBD, c.reg.a);
    assert_eq!(c.flags(), SF | VF | NF | CF); // SBC A,H (0x7D - 0xC0)
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xFD, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // SBC A,L (0xBD - 0xBF - carry ) should set HF
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFB, c.reg.a);
    assert_eq!(c.flags(), SF | NF); // SBC A,0x01
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFD, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // SBC A,0xFE
}

#[test]
fn sbc_ixyh_ixyl_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/sbc_ixyh_ixyl.bin", 0).unwrap();
    c.execute(&mut b);
    c.execute(&mut b);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // SUB A,A
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // SBC A,IXH
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x06, c.reg.a);
    assert_eq!(c.flags(), NF); // SBC A,IXL
    c.execute(&mut b);
    c.execute(&mut b);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // SUB A,A
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // SBC A,IYH
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x06, c.reg.a);
    assert_eq!(c.flags(), NF); // SBC A,IYL
}

#[test]
fn sbc_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x41);
    b.write_byte(0x1001, 0x61);
    b.write_byte(0x1002, 0x81);
    b.load_bin("bin/sbc_i_hl_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1000, c.reg.get_hl());
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1000, c.reg.get_ix(),);
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x1003, c.reg.get_iy());
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xBF, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x5D, c.reg.a);
    assert_eq!(c.flags(), VF | NF);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xFC, c.reg.a);
    assert_eq!(c.flags(), SF | NF | CF);
}

#[test]
fn or_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/or_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | PF); // OR A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.flags(), 0); // OR B
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x03, c.reg.a);
    assert_eq!(c.flags(), PF); // OR C
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x07, c.reg.a);
    assert_eq!(c.flags(), 0); // OR D
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x0F, c.reg.a);
    assert_eq!(c.flags(), PF); // OR E
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x1F, c.reg.a);
    assert_eq!(c.flags(), 0); // OR H
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x3F, c.reg.a);
    assert_eq!(c.flags(), PF); // OR L
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x7F, c.reg.a);
    assert_eq!(c.flags(), 0); // OR 0x40
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | PF); // OR 0x80
}

#[test]
fn xor_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/xor_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | PF); // XOR A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.flags(), 0); // XOR B
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x02, c.reg.a);
    assert_eq!(c.flags(), 0); // XOR C
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x05, c.reg.a);
    assert_eq!(c.flags(), PF); // XOR D
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x0A, c.reg.a);
    assert_eq!(c.flags(), PF); // XOR E
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x15, c.reg.a);
    assert_eq!(c.flags(), 0); // XOR H
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x2A, c.reg.a);
    assert_eq!(c.flags(), 0); // XOR L
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x55, c.reg.a);
    assert_eq!(c.flags(), PF); // XOR 0x7F
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xAA, c.reg.a);
    assert_eq!(c.flags(), SF | PF); // XOR 0xFF
}

#[test]
fn or_xor_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x41);
    b.write_byte(0x1001, 0x62);
    b.write_byte(0x1002, 0x84);
    b.load_bin("bin/or_xor_i_hl_ix_iy.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x41, c.reg.a);
    assert_eq!(c.flags(), PF); // OR (HL)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x63, c.reg.a);
    assert_eq!(c.flags(), PF); // OR (IX+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xE7, c.reg.a);
    assert_eq!(c.flags(), SF | PF); // OR (IY-1)
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xA6, c.reg.a);
    assert_eq!(c.flags(), SF | PF); // XOR (HL)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xC4, c.reg.a);
    assert_eq!(c.flags(), SF); // XOR (IX+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x40, c.reg.a);
    assert_eq!(c.flags(), 0); // XOR (IY-1)
}

#[test]
fn and_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/and_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.flags(), HF); // AND B
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | PF); // OR 0xFF
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x03, c.reg.a);
    assert_eq!(c.flags(), HF | PF); // AND C
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | PF); // OR 0xFF
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x04, c.reg.a);
    assert_eq!(c.flags(), HF); // AND D
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | PF); // OR 0xFF
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x08, c.reg.a);
    assert_eq!(c.flags(), HF); // AND E
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | PF); // OR 0xFF
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x10, c.reg.a);
    assert_eq!(c.flags(), HF); // AND H
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | PF); // OR 0xFF
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x20, c.reg.a);
    assert_eq!(c.flags(), HF); // AND L
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | PF); // OR 0xFF
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x40, c.reg.a);
    assert_eq!(c.flags(), HF); // AND 0x40
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF | PF); // OR 0xFF
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xAA, c.reg.a);
    assert_eq!(c.flags(), SF | HF | PF); // AND 0xAA
}

#[test]
fn and_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0xFE);
    b.write_byte(0x1001, 0xAA);
    b.write_byte(0x1002, 0x99);
    b.load_bin("bin/and_i_hl_ix_iy.bin", 0).unwrap();
    for _ in 0..4 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFE, c.reg.a);
    assert_eq!(c.flags(), SF | HF); // AND (HL)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xAA, c.reg.a);
    assert_eq!(c.flags(), SF | HF | PF); // AND (IX+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x88, c.reg.a);
    assert_eq!(c.flags(), SF | HF | PF); // AND (IY-1)
}

#[test]
fn inc_dec_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/inc_dec_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.flags(), 0); // INC A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // DEC A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.b);
    assert_eq!(c.flags(), ZF | HF); // INC B
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xFF, c.reg.b);
    assert_eq!(c.flags(), SF | HF | NF); // DEC B
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x10, c.reg.c);
    assert_eq!(c.flags(), HF); // INC C
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x0F, c.reg.c);
    assert_eq!(c.flags(), HF | NF); // DEC C
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x0F, c.reg.d);
    assert_eq!(c.flags(), 0); // INC D
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x0E, c.reg.d);
    assert_eq!(c.flags(), NF); // DEC D
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF | CF); // CP 0x01   set carry flag (should be preserved)
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x80, c.reg.e);
    assert_eq!(c.flags(), SF | HF | VF | CF); // INC E
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x7F, c.reg.e);
    assert_eq!(c.flags(), HF | VF | NF | CF); // DEC E
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x3F, c.reg.h);
    assert_eq!(c.flags(), CF); // INC H
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x3E, c.reg.h);
    assert_eq!(c.flags(), NF | CF); // DEC H
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x24, c.reg.l);
    assert_eq!(c.flags(), CF); // INC L
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x23, c.reg.l);
    assert_eq!(c.flags(), NF | CF); // DEC L
}

#[test]
fn inc_dec_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x00);
    b.write_byte(0x1001, 0x3F);
    b.write_byte(0x1002, 0x7F);
    b.load_bin("bin/inc_dec_i_hl_ix_iy.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0xFF, b.read_byte(0x1000));
    assert_eq!(c.flags(), SF | HF | NF); // DEC (HL)
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x00, b.read_byte(0x1000));
    assert_eq!(c.flags(), ZF | HF); // INC (HL)
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x40, b.read_byte(0x1001));
    assert_eq!(c.flags(), HF); // INC (IX+1)
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x3F, b.read_byte(0x1001));
    assert_eq!(c.flags(), HF | NF); // DEC (IX+1)
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x80, b.read_byte(0x1002));
    assert_eq!(c.flags(), SF | HF | VF); // INC (IY-1)
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x7F, b.read_byte(0x1002));
    assert_eq!(c.flags(), HF | PF | NF); // DEC (IY-1)
}

#[test]
fn inc_dec_ss_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/inc_dec_ss_ix_iy.bin", 0).unwrap();
    for _ in 0..6 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0xFFFF, c.reg.get_bc()); // DEC BC
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x0000, c.reg.get_bc()); // INC BC
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x0000, c.reg.get_de()); // INC DE
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0xFFFF, c.reg.get_de()); // DEC DE
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x0100, c.reg.get_hl()); // INC HL
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x00FF, c.reg.get_hl()); // DEC HL
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x1112, c.reg.sp); // INC SP
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x1111, c.reg.sp); // DEC SP
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1000, c.reg.get_ix(),); // INC IX
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0FFF, c.reg.get_ix(),); // DEC IX
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1235, c.reg.get_iy()); // INC IY
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1234, c.reg.get_iy()); // DEC IY
}

#[test]
fn djnz_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/djnz.bin", 0x0204).unwrap();
    c.reg.pc = 0x0204;
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x03, c.reg.b);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(0x02, c.reg.b);
    assert_eq!(0x0207, c.reg.pc);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x02, c.reg.a);
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(0x01, c.reg.b);
    assert_eq!(0x0207, c.reg.pc);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x03, c.reg.a);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x00, c.reg.b);
    assert_eq!(0x020A, c.reg.pc);
}

#[test]
fn jr_cc_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/jr_cc.bin", 0x0204).unwrap();
    c.reg.pc = 0x0204;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x0207, c.reg.pc);
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(0x020A, c.reg.pc);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x020E, c.reg.pc);
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(0x0211, c.reg.pc);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFE, c.reg.a);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x0215, c.reg.pc);
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(0x0218, c.reg.pc);
}

#[test]
fn ld_i_hl_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ld_i_hl_r.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1000, c.reg.get_hl());
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x12, c.reg.a);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x12, b.read_byte(0x1000));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x13, c.reg.b);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x13, b.read_byte(0x1000));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x14, c.reg.c);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x14, b.read_byte(0x1000));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x15, c.reg.d);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x15, b.read_byte(0x1000));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x16, c.reg.e);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x16, b.read_byte(0x1000));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x10, b.read_byte(0x1000));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, b.read_byte(0x1000));
}

#[test]
fn ld_a_i_bc_de_nn_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ld_a_i_bc_de_nn.bin", 0).unwrap();
    b.write_byte(0x1000, 0x11);
    b.write_byte(0x1001, 0x22);
    b.write_byte(0x1002, 0x33);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1000, c.reg.get_bc());
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1001, c.reg.get_de());
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x11, c.reg.a);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x22, c.reg.a);
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(0x33, c.reg.a);
}

#[test]
fn inc_dec_ss_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/inc_dec_ss.bin", 0).unwrap();
    for _ in 0..4 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0xFFFF, c.reg.get_bc());
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x0000, c.reg.get_bc());
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x0000, c.reg.get_de());
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0xFFFF, c.reg.get_de());
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x0100, c.reg.get_hl());
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x00FF, c.reg.get_hl());
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x1112, c.reg.sp);
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(0x1111, c.reg.sp);
}

#[test]
fn ld_i_bc_de_nn_a_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ld_i_bc_de_nn_a.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1000, c.reg.get_bc()); // LD BC,0x1000
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1001, c.reg.get_de()); // LD DE,0x1001
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x77, c.reg.a); // LD A,0x77
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x77, b.read_byte(0x1000)); // LD (BC),A
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x77, b.read_byte(0x1001)); // LD (DE),A
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(0x77, b.read_byte(0x1002)); // LD (0x1002),A
}

#[test]
fn rlca_rla_rrca_rra_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/rlca_rla_rrca_rra.bin", 0).unwrap();
    c.reg.flags.set_from_byte(0xFF);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xA0, c.reg.a); // LD A,0xA0
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x41, c.reg.a); // RLCA
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x82, c.reg.a); // RLCA
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x41, c.reg.a); // RRCA
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xA0, c.reg.a); // RRCA
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x41, c.reg.a); // RLA
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x83, c.reg.a); // RLA
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x41, c.reg.a); // RRA
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xA0, c.reg.a); // RRA
}

#[test]
fn daa_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/daa.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x15, c.reg.a); // LD A,0x15
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x27, c.reg.b); // LD B,0x27
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x3C, c.reg.a);
    assert_eq!(c.flags(), 0); // ADD A,B
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x42, c.reg.a);
    assert_eq!(c.flags(), HF | PF); // DAA
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x1B, c.reg.a);
    assert_eq!(c.flags(), HF | NF); // SUB B
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x15, c.reg.a);
    assert_eq!(c.flags(), NF); // DAA
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x90, c.reg.a);
    assert_eq!(c.flags(), NF); // LD A,0x90
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x15, c.reg.b);
    assert_eq!(c.flags(), NF); // LD B,0x15
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xA5, c.reg.a);
    assert_eq!(c.flags(), SF); // ADD A,B
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x05, c.reg.a);
    assert_eq!(c.flags(), PF | CF); // DAA
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xF0, c.reg.a);
    assert_eq!(c.flags(), SF | NF | CF); // SUB B
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x90, c.reg.a);
    assert_eq!(c.flags(), SF | PF | NF | CF); // DAA
}

#[test]
fn cpl_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/cpl.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // SUB A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), ZF | HF | NF); // CPL
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | HF | NF); // CPL
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xAA, c.reg.a);
    assert_eq!(c.flags(), SF); // ADD A,0xAA
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x55, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF); // CPL
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0xAA, c.reg.a);
    assert_eq!(c.flags(), SF | HF | NF); // CPL
}

#[test]
fn ccf_scf_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ccf_scf.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | NF); // SUB A
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | CF); // SCF
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | HF); // CCF
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x34, c.reg.a);
    assert_eq!(c.flags(), HF | NF | CF); // SUB 0xCC
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x34, c.reg.a);
    assert_eq!(c.flags(), HF); // CCF
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x34, c.reg.a);
    assert_eq!(c.flags(), CF); // SCF
}

#[test]
fn call_ret_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/call_ret.bin", 0x0204).unwrap();
    c.reg.pc = 0x0204;
    assert_eq!(c.execute(&mut b), 17);
    assert_eq!(0x020A, c.reg.pc);
    assert_eq!(0xFFFE, c.reg.sp);
    assert_eq!(0x0207, b.read_word(0xFFFE));
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0207, c.reg.pc);
    assert_eq!(0x0000, c.reg.sp);
    assert_eq!(c.execute(&mut b), 17);
    assert_eq!(0x020A, c.reg.pc);
    assert_eq!(0xFFFE, c.reg.sp);
    assert_eq!(0x020A, b.read_word(0xFFFE));
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x020A, c.reg.pc);
    assert_eq!(0x0000, c.reg.sp);
}

#[test]
fn call_cc_ret_cc_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/call_cc_ret_cc.bin", 0x0204).unwrap();
    c.reg.pc = 0x0204;
    c.reg.sp = 0x0100;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0208, c.reg.pc);
    assert_eq!(c.execute(&mut b), 17);
    assert_eq!(0x0229, c.reg.pc);
    assert_eq!(c.execute(&mut b), 5);
    assert_eq!(0x022A, c.reg.pc);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x020B, c.reg.pc);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0210, c.reg.pc);
    assert_eq!(c.execute(&mut b), 17);
    assert_eq!(0x022B, c.reg.pc);
    assert_eq!(c.execute(&mut b), 5);
    assert_eq!(0x022C, c.reg.pc);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x0213, c.reg.pc);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x02, c.reg.a);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0217, c.reg.pc);
    assert_eq!(c.execute(&mut b), 17);
    assert_eq!(0x022D, c.reg.pc);
    assert_eq!(c.execute(&mut b), 5);
    assert_eq!(0x022E, c.reg.pc);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x021A, c.reg.pc);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x021F, c.reg.pc);
    assert_eq!(c.execute(&mut b), 17);
    assert_eq!(0x022F, c.reg.pc);
    assert_eq!(c.execute(&mut b), 5);
    assert_eq!(0x0230, c.reg.pc);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x0222, c.reg.pc);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0225, c.reg.pc);
    assert_eq!(c.execute(&mut b), 17);
    assert_eq!(0x0231, c.reg.pc);
    assert_eq!(c.execute(&mut b), 5);
    assert_eq!(0x0232, c.reg.pc);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x0228, c.reg.pc);
}

#[test]
fn halt_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/halt.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x0000, c.reg.pc);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x0000, c.reg.pc);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x0000, c.reg.pc);
}

#[test]
fn ex_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ex.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1234, c.reg.get_hl());
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x5678, c.reg.get_de());
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x1234, c.reg.get_de());
    assert_eq!(0x5678, c.reg.get_hl());
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x1100, c.reg.get_af());
    assert_eq!(0x0000, c.alt.get_af());
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x0000, c.reg.get_af());
    assert_eq!(0x1100, c.alt.get_af());
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x2200, c.reg.get_af());
    assert_eq!(0x1100, c.alt.get_af());
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x1100, c.reg.get_af());
    assert_eq!(0x2200, c.alt.get_af());
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x9ABC, c.reg.get_bc());
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x0000, c.reg.get_hl());
    assert_eq!(0x5678, c.alt.get_hl());
    assert_eq!(0x0000, c.reg.get_de());
    assert_eq!(0x1234, c.alt.get_de());
    assert_eq!(0x0000, c.reg.get_bc());
    assert_eq!(0x9ABC, c.alt.get_bc());
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1111, c.reg.get_hl());
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x2222, c.reg.get_de());
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x3333, c.reg.get_bc());
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x5678, c.reg.get_hl());
    assert_eq!(0x1111, c.alt.get_hl());
    assert_eq!(0x1234, c.reg.get_de());
    assert_eq!(0x2222, c.alt.get_de());
    assert_eq!(0x9ABC, c.reg.get_bc());
    assert_eq!(0x3333, c.alt.get_bc());
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0100, c.reg.sp);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x1234, b.read_word(0x00FE));
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x1234, c.reg.get_hl());
    assert_eq!(0x5678, b.read_word(0x00FE));
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x8899, c.reg.get_ix(),);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x5678, c.reg.get_ix(),);
    assert_eq!(0x8899, b.read_word(0x00FE));
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x6677, c.reg.get_iy());
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x8899, c.reg.get_iy());
    assert_eq!(0x6677, b.read_word(0x00FE));
}

#[test]
fn jp_cc_nn_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/jp_cc_nn.bin", 0x0204).unwrap();
    c.reg.pc = 0x0204;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | NF);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0208, c.reg.pc);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x020C, c.reg.pc);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0211, c.reg.pc);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0215, c.reg.pc);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x02, c.reg.a);
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0219, c.reg.pc);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x021D, c.reg.pc);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFF, c.reg.a);
    assert_eq!(c.flags(), SF);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0222, c.reg.pc);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0226, c.reg.pc);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x022D, c.reg.pc);
}

#[test]
fn jp_jr_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/jp_jr.bin", 0x0204).unwrap();
    c.reg.pc = 0x0204;
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0216, c.reg.get_hl());
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x0219, c.reg.get_ix(),);
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x0221, c.reg.get_iy());
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0214, c.reg.pc);
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(0x0212, c.reg.pc);
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(0x0218, c.reg.pc);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x0216, c.reg.pc);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x0219, c.reg.pc);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x0221, c.reg.pc);
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(0x021B, c.reg.pc);
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(0x0223, c.reg.pc);
}

#[test]
fn ldi_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x02);
    b.write_byte(0x1002, 0x03);
    b.load_bin("bin/ldi.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x1001, c.reg.get_hl());
    assert_eq!(0x2001, c.reg.get_de());
    assert_eq!(0x0002, c.reg.get_bc());
    assert_eq!(0x01, b.read_byte(0x2000));
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x1002, c.reg.get_hl());
    assert_eq!(0x2002, c.reg.get_de());
    assert_eq!(0x0001, c.reg.get_bc());
    assert_eq!(0x02, b.read_byte(0x2001));
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x1003, c.reg.get_hl());
    assert_eq!(0x2003, c.reg.get_de());
    assert_eq!(0x0000, c.reg.get_bc());
    assert_eq!(0x03, b.read_byte(0x2002));
    assert_eq!(c.flags(), 0);
}

#[test]
fn ldir_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x02);
    b.write_byte(0x1002, 0x03);
    b.load_bin("bin/ldir.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }
    run_block(&mut c, &mut b);
    assert_eq!(0x1003, c.reg.get_hl());
    assert_eq!(0x2003, c.reg.get_de());
    assert_eq!(0x0000, c.reg.get_bc());
    assert_eq!(0x03, b.read_byte(0x2002));
    assert_eq!(c.flags(), 0);
    c.execute(&mut b);
    assert_eq!(0x33, c.reg.a);
}

#[test]
fn ldir_returns_total_cycles() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB0);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x02);
    b.write_byte(0x1002, 0x03);
    c.reg.set_hl(0x1000);
    c.reg.set_de(0x2000);
    c.reg.set_bc(0x0003);

    assert_eq!(run_block(&mut c, &mut b), 58);
    assert_eq!(c.reg.pc, 0x0002);
    assert_eq!(c.reg.get_hl(), 0x1003);
    assert_eq!(c.reg.get_de(), 0x2003);
    assert_eq!(c.reg.get_bc(), 0x0000);
    assert_eq!(b.read_byte(0x2000), 0x01);
    assert_eq!(b.read_byte(0x2001), 0x02);
    assert_eq!(b.read_byte(0x2002), 0x03);
    assert_eq!(c.flags(), 0);
}

#[test]
fn ldir_with_zero_bc_repeats_64kb() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB0);
    c.reg.set_hl(0x0000);
    c.reg.set_de(0x0000);
    c.reg.set_bc(0x0000);

    assert_eq!(run_block(&mut c, &mut b), 16 + (21_u32 * 0xFFFF));
    assert_eq!(c.reg.pc, 0x0002);
    assert_eq!(c.reg.get_hl(), 0x0000);
    assert_eq!(c.reg.get_de(), 0x0000);
    assert_eq!(c.reg.get_bc(), 0x0000);
    assert_eq!(c.flags(), 0);
}

#[test]
fn ldd_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x02);
    b.write_byte(0x1002, 0x03);
    b.load_bin("bin/ldd.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x1001, c.reg.get_hl());
    assert_eq!(0x2001, c.reg.get_de());
    assert_eq!(0x0002, c.reg.get_bc());
    assert_eq!(0x03, b.read_byte(0x2002));
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x1000, c.reg.get_hl());
    assert_eq!(0x2000, c.reg.get_de());
    assert_eq!(0x0001, c.reg.get_bc());
    assert_eq!(0x02, b.read_byte(0x2001));
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x0FFF, c.reg.get_hl());
    assert_eq!(0x1FFF, c.reg.get_de());
    assert_eq!(0x0000, c.reg.get_bc());
    assert_eq!(0x01, b.read_byte(0x2000));
    assert_eq!(c.flags(), 0);
}

#[test]
fn lddr_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x02);
    b.write_byte(0x1002, 0x03);
    b.load_bin("bin/lddr.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }
    run_block(&mut c, &mut b);
    assert_eq!(0x0FFF, c.reg.get_hl());
    assert_eq!(0x1FFF, c.reg.get_de());
    assert_eq!(0x0000, c.reg.get_bc());
    assert_eq!(0x01, b.read_byte(0x2000));
    assert_eq!(c.flags(), 0);
    c.execute(&mut b);
    assert_eq!(0x33, c.reg.a);
}

#[test]
fn lddr_returns_total_cycles() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB8);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x02);
    b.write_byte(0x1002, 0x03);
    c.reg.set_hl(0x1002);
    c.reg.set_de(0x2002);
    c.reg.set_bc(0x0003);

    assert_eq!(run_block(&mut c, &mut b), 58);
    assert_eq!(c.reg.pc, 0x0002);
    assert_eq!(c.reg.get_hl(), 0x0FFF);
    assert_eq!(c.reg.get_de(), 0x1FFF);
    assert_eq!(c.reg.get_bc(), 0x0000);
    assert_eq!(b.read_byte(0x2000), 0x01);
    assert_eq!(b.read_byte(0x2001), 0x02);
    assert_eq!(b.read_byte(0x2002), 0x03);
    assert_eq!(c.flags(), 0);
}

#[test]
fn lddr_with_zero_bc_repeats_64kb() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB8);
    b.write_byte(0xFFFF, 0xAA);
    c.reg.set_hl(0xFFFF);
    c.reg.set_de(0xFFFF);
    c.reg.set_bc(0x0000);

    assert_eq!(run_block(&mut c, &mut b), 16 + (21_u32 * 0xFFFF));
    assert_eq!(c.reg.pc, 0x0002);
    assert_eq!(c.reg.get_hl(), 0xFFFF);
    assert_eq!(c.reg.get_de(), 0xFFFF);
    assert_eq!(c.reg.get_bc(), 0x0000);
    assert_eq!(b.read_byte(0xFFFF), 0xAA);
    assert_eq!(c.flags(), 0);
}

#[test]
fn cpi_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x02);
    b.write_byte(0x1002, 0x03);
    b.write_byte(0x1003, 0x04);
    b.load_bin("bin/cpi.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x1001, c.reg.get_hl());
    assert_eq!(0x0003, c.reg.get_bc());
    assert_eq!(c.flags(), PF | NF);
    let f = c.flags() | CF;
    c.reg.flags.set_from_byte(f);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x1002, c.reg.get_hl());
    assert_eq!(0x0002, c.reg.get_bc());
    assert_eq!(c.flags(), PF | NF | CF);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x1003, c.reg.get_hl());
    assert_eq!(0x0001, c.reg.get_bc());
    assert_eq!(c.flags(), ZF | PF | NF | CF);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x1004, c.reg.get_hl());
    assert_eq!(0x0000, c.reg.get_bc());
    assert_eq!(c.flags(), SF | HF | NF | CF);
}

#[test]
fn cpir_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x02);
    b.write_byte(0x1002, 0x03);
    b.write_byte(0x1003, 0x04);
    b.load_bin("bin/cpir.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }

    run_block(&mut c, &mut b);
    assert_eq!(0x1003, c.reg.get_hl());
    assert_eq!(0x0001, c.reg.get_bc());
    assert_eq!(c.flags(), ZF | PF | NF);

    c.execute(&mut b);
    assert_eq!(0x1004, c.reg.get_hl());
    assert_eq!(0x0000, c.reg.get_bc());
    assert_eq!(c.flags(), SF | HF | NF);
}

#[test]
fn cpd_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x02);
    b.write_byte(0x1002, 0x03);
    b.write_byte(0x1003, 0x04);
    b.load_bin("bin/cpd.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x1002, c.reg.get_hl());
    assert_eq!(0x0003, c.reg.get_bc());
    assert_eq!(c.flags(), SF | HF | PF | NF);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x1001, c.reg.get_hl());
    assert_eq!(0x0002, c.reg.get_bc());
    assert_eq!(c.flags(), ZF | PF | NF);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x1000, c.reg.get_hl());
    assert_eq!(0x0001, c.reg.get_bc());
    assert_eq!(c.flags(), PF | NF);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x0FFF, c.reg.get_hl());
    assert_eq!(0x0000, c.reg.get_bc());
    assert_eq!(c.flags(), NF);
}

#[test]
fn add_adc_sbc_16_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/add_adc_sbc_16.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x00FC, c.reg.get_hl());
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0008, c.reg.get_bc());
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0xFFFF, c.reg.get_de());
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x0104, c.reg.get_hl());
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x0103, c.reg.get_hl());
    assert_eq!(c.flags(), HF | CF);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x010C, c.reg.get_hl());
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x0218, c.reg.get_hl());
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(0x0217, c.reg.get_hl());
    assert_eq!(c.flags(), HF | CF);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x020E, c.reg.get_hl());
    assert_eq!(c.flags(), NF);
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x00FC, c.reg.get_ix(),);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1000, c.reg.sp);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x0104, c.reg.get_ix(),);
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x0103, c.reg.get_ix(),);
    assert_eq!(c.flags(), HF | CF);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x0206, c.reg.get_ix(),);
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x1206, c.reg.get_ix(),);
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0xFFFF, c.reg.get_iy());
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x0007, c.reg.get_iy());
    assert_eq!(c.flags(), HF | CF);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x0006, c.reg.get_iy());
    assert_eq!(c.flags(), HF | CF);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x000C, c.reg.get_iy());
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x100C, c.reg.get_iy());
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x7FFF, c.reg.get_hl());
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0001, c.reg.get_bc());
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x8000, c.reg.get_hl());
    assert_eq!(c.flags(), SF | HF | PF);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x7FFF, c.reg.get_hl());
    assert_eq!(c.flags(), NF | HF | PF);
}

#[test]
fn ld_inn_hl_dd_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ld_inn_hl_dd_ix_iy.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x0201, c.reg.get_hl()); // LD HL,0x0201
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x0201, b.read_word(0x1000)); // LD (0x1000),HL
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1234, c.reg.get_bc()); // LD BC,0x1234
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(0x1234, b.read_word(0x1002)); // LD (0x1002),BC
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x5678, c.reg.get_de()); // LD DE,0x5678
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(0x5678, b.read_word(0x1004)); // LD (0x1004),DE
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x9ABC, c.reg.get_hl()); // LD HL,0x9ABC
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(0x9ABC, b.read_word(0x1006)); // LD (0x1006),HL
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1368, c.reg.sp); // LD SP,0x1368
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(0x1368, b.read_word(0x1008)); // LD (0x1008),SP
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x4321, c.reg.get_ix(),); // LD IX,0x4321
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(0x4321, b.read_word(0x100A)); // LD (0x100A),IX
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(0x8765, c.reg.get_iy()); // LD IY,0x8765
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(0x8765, b.read_word(0x100C)); // LD (0x100C),IY
}

#[test]
fn ld_a_ir_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFB); // EI
    b.write_byte(0x0001, 0xED); // LD A,I
    b.write_byte(0x0002, 0x57);
    b.write_byte(0x0003, 0x97); // SUB A
    b.write_byte(0x0004, 0xED); // LD A,R
    b.write_byte(0x0005, 0x5F);
    c.reg.r = 0x34;
    c.reg.i = 0x1;
    c.reg.flags.c = true;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.execute(&mut b), 9);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.flags(), PF | CF);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | NF);
    assert_eq!(c.execute(&mut b), 9);
    assert_eq!(0x34, c.reg.a);
    assert_eq!(c.flags(), PF);
}

#[test]
fn ld_ir_a_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/ld_ir_a.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x45, c.reg.a);
    assert_eq!(c.execute(&mut b), 9);
    assert_eq!(0x45, c.reg.i);
    assert_eq!(c.execute(&mut b), 9);
    assert_eq!(0x45, c.reg.r);
}

#[test]
fn ld_a_i_interrupt_pending_clears_parity() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFB); // EI
    b.write_byte(0x0001, 0xED); // LD A,I
    b.write_byte(0x0002, 0x57);
    b.write_byte(0x0003, 0x00); // NOP
    c.reg.i = 0x80;
    c.reg.sp = 0x2000;
    c.execute(&mut b);
    c.int_request(0xCF);
    assert_eq!(c.execute(&mut b), 9);
    assert_eq!(c.reg.a, 0x80);
    // P/V must be cleared when an interrupt is pending during the EI delay window.
    assert_eq!(c.flags() & PF, 0);
    // IM 0 acknowledge: RST 08 fetched from the data bus, 11 + 2 wait states.
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(c.reg.pc, 0x0008);
}

#[test]
fn ld_a_r_interrupt_pending_clears_parity() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFB); // EI
    b.write_byte(0x0001, 0xED); // LD A,R
    b.write_byte(0x0002, 0x5F);
    b.write_byte(0x0003, 0x00); // NOP
    c.reg.r = 0x44;
    c.reg.sp = 0x2000;
    c.execute(&mut b);
    c.int_request(0xCF);
    assert_eq!(c.execute(&mut b), 9);
    assert_eq!(c.reg.a, 0x44);
    // P/V must be cleared when an interrupt is pending during the EI delay window.
    assert_eq!(c.flags() & PF, 0);
    // IM 0 acknowledge: RST 08 fetched from the data bus, 11 + 2 wait states.
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(c.reg.pc, 0x0008);
}

#[test]
fn rlc_rl_rrc_rr_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/rlc_rl_rrc_rr_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x80, c.reg.a);
    assert_eq!(c.flags(), SF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.flags(), CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFF, c.reg.b);
    assert_eq!(c.flags(), SF | PF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFF, c.reg.b);
    assert_eq!(c.flags(), SF | PF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x06, c.reg.c);
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x03, c.reg.c);
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFD, c.reg.d);
    assert_eq!(c.flags(), SF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFE, c.reg.d);
    assert_eq!(c.flags(), SF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x88, c.reg.e);
    assert_eq!(c.flags(), SF | PF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x11, c.reg.e);
    assert_eq!(c.flags(), PF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x7E, c.reg.h);
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x3F, c.reg.h);
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xE0, c.reg.l);
    assert_eq!(c.flags(), SF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x70, c.reg.l);
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | PF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x7F, c.reg.b);
    assert_eq!(c.flags(), CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFF, c.reg.b);
    assert_eq!(c.flags(), SF | PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x06, c.reg.c);
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x03, c.reg.c);
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFC, c.reg.d);
    assert_eq!(c.flags(), SF | PF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFE, c.reg.d);
    assert_eq!(c.flags(), SF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x08, c.reg.e);
    assert_eq!(c.flags(), CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x11, c.reg.e);
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x7E, c.reg.h);
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x3F, c.reg.h);
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xE0, c.reg.l);
    assert_eq!(c.flags(), SF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x70, c.reg.l);
    assert_eq!(c.flags(), 0);
}

#[test]
fn rrc_rlc_rr_rl_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0xFF);
    b.write_byte(0x1002, 0x11);
    b.load_bin("bin/rrc_rlc_rr_rl_i_hl_ix_iy.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x80, b.read_byte(0x1000));
    assert_eq!(c.flags(), SF | CF); // RRC (HL)
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x80, c.reg.a); // LD A,(HL)
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x01, b.read_byte(0x1000));
    assert_eq!(c.flags(), CF); // RLC (HL)
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x01, c.reg.a); // LD A,(HL)
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0xFF, b.read_byte(0x1001));
    assert_eq!(c.flags(), SF | PF | CF); // RRC (IX+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xFF, c.reg.a); // LD A,(IX+1)
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0xFF, b.read_byte(0x1001));
    assert_eq!(c.flags(), SF | PF | CF); // RLC (IX+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xFF, c.reg.a); // LD A,(IX+1)
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x88, b.read_byte(0x1002));
    assert_eq!(c.flags(), SF | PF | CF); // RRC (IY-1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x88, c.reg.a); // LD A,(IY-1)
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x11, b.read_byte(0x1002));
    assert_eq!(c.flags(), PF | CF); // RLC (IY-1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x11, c.reg.a); // LD A,(IY-1)
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x80, b.read_byte(0x1000));
    assert_eq!(c.flags(), SF | CF); // RR (HL)
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x80, c.reg.a); // LD A,(HL)
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x01, b.read_byte(0x1000));
    assert_eq!(c.flags(), CF); // RL (HL)
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x01, c.reg.a); // LD A,(HL)
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0xFF, b.read_byte(0x1001));
    assert_eq!(c.flags(), SF | PF | CF); // RR (IX+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xFF, c.reg.a); // LD A,(IX+1)
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0xFF, b.read_byte(0x1001));
    assert_eq!(c.flags(), SF | PF | CF); // RL (IX+1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xFF, c.reg.a); // LD A,(IX+1)
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x23, b.read_byte(0x1002));
    assert_eq!(c.flags(), 0); // RL (IY-1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x23, c.reg.a); // LD A,(IY-1)
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x11, b.read_byte(0x1002));
    assert_eq!(c.flags(), PF | CF); // RR (IY-1)
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x11, c.reg.a); // LD A,(IY-1)
}

#[test]
fn sla_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/sla_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x02, c.reg.a);
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x00, c.reg.b);
    assert_eq!(c.flags(), ZF | PF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x54, c.reg.c);
    assert_eq!(c.flags(), CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFC, c.reg.d);
    assert_eq!(c.flags(), SF | PF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFE, c.reg.e);
    assert_eq!(c.flags(), SF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x22, c.reg.h);
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x00, c.reg.l);
    assert_eq!(c.flags(), ZF | PF);
}

#[test]
fn sra_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/sra_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | PF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xC0, c.reg.b);
    assert_eq!(c.flags(), SF | PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xD5, c.reg.c);
    assert_eq!(c.flags(), SF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0xFF, c.reg.d);
    assert_eq!(c.flags(), SF | PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x3F, c.reg.e);
    assert_eq!(c.flags(), PF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x08, c.reg.h);
    assert_eq!(c.flags(), CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x00, c.reg.l);
    assert_eq!(c.flags(), ZF | PF);
}

#[test]
fn srl_r_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/srl_r.bin", 0).unwrap();
    for _ in 0..7 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.flags(), ZF | PF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x40, c.reg.b);
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x55, c.reg.c);
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x7F, c.reg.d);
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x3F, c.reg.e);
    assert_eq!(c.flags(), PF | CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x08, c.reg.h);
    assert_eq!(c.flags(), CF);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(0x00, c.reg.l);
    assert_eq!(c.flags(), ZF | PF);
}

#[test]
fn sla_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x80);
    b.write_byte(0x1002, 0xAA);
    b.load_bin("bin/sla_i_hl_ix_iy.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x02, b.read_byte(0x1000));
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x02, c.reg.a);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x00, b.read_byte(0x1001));
    assert_eq!(c.flags(), ZF | PF | CF);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x54, b.read_byte(0x1002));
    assert_eq!(c.flags(), CF);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x54, c.reg.a);
}

#[test]
fn sra_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x80);
    b.write_byte(0x1002, 0xAA);
    b.load_bin("bin/sra_i_hl_ix_iy.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x00, b.read_byte(0x1000));
    assert_eq!(c.flags(), ZF | PF | CF);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0xC0, b.read_byte(0x1001));
    assert_eq!(c.flags(), SF | PF);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xC0, c.reg.a);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0xD5, b.read_byte(0x1002));
    assert_eq!(c.flags(), SF);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0xD5, c.reg.a);
}

#[test]
fn srl_i_hl_ix_iy_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x1000, 0x01);
    b.write_byte(0x1001, 0x80);
    b.write_byte(0x1002, 0xAA);
    b.load_bin("bin/srl_i_hl_ix_iy.bin", 0).unwrap();
    for _ in 0..3 {
        c.execute(&mut b);
    }
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(0x00, b.read_byte(0x1000));
    assert_eq!(c.flags(), ZF | PF | CF);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x40, b.read_byte(0x1001));
    assert_eq!(c.flags(), 0);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x40, c.reg.a);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(0x55, b.read_byte(0x1002));
    assert_eq!(c.flags(), PF);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(0x55, c.reg.a);
}

#[test]
fn rld_rrd_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/rld_rrd.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x12, c.reg.a);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x1000, c.reg.get_hl());
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x34, b.read_byte(0x1000));
    assert_eq!(c.execute(&mut b), 18);
    assert_eq!(0x14, c.reg.a);
    assert_eq!(0x23, b.read_byte(0x1000));
    assert_eq!(c.execute(&mut b), 18);
    assert_eq!(0x12, c.reg.a);
    assert_eq!(0x34, b.read_byte(0x1000));
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x34, c.reg.a);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0xFE, c.reg.a);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x00, b.read_byte(0x1000));
    assert_eq!(c.execute(&mut b), 18);
    assert_eq!(0xF0, c.reg.a);
    assert_eq!(0x0E, b.read_byte(0x1000));
    assert_eq!(c.flags(), SF | PF);
    assert_eq!(c.execute(&mut b), 18);
    assert_eq!(0xFE, c.reg.a);
    assert_eq!(0x00, b.read_byte(0x1000));
    assert_eq!(c.flags(), SF);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(0x00, b.read_byte(0x1000));
    c.reg.flags.set_from_byte(CF);
    assert_eq!(c.execute(&mut b), 18);
    assert_eq!(0x00, c.reg.a);
    assert_eq!(0x01, b.read_byte(0x1000));
    assert_eq!(c.flags(), ZF | PF | CF);
    assert_eq!(c.execute(&mut b), 18);
    assert_eq!(0x01, c.reg.a);
    assert_eq!(0x00, b.read_byte(0x1000));
    assert_eq!(c.flags(), CF);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(0x00, c.reg.a);
}

#[test]
fn ld_inn_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x63);
    b.write_byte(0x0002, 0x06);
    b.write_byte(0x0003, 0x10);
    c.reg.set_hl(0x9ABC);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(0x9ABC, b.read_word(0x1006));
}

#[test]
fn ld_b() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.b = 0x11;
    c.reg.c = 0x15;
    c.reg.d = 0x1F;
    c.reg.e = 0x21;
    c.reg.h = 0x25;
    c.reg.l = 0x2F;
    b.write_byte(0x252f, 0x31);
    c.reg.a = 0x3F;
    b.write_byte(0x0000, 0x40);
    b.write_byte(0x0001, 0x41);
    b.write_byte(0x0002, 0x42);
    b.write_byte(0x0003, 0x43);
    b.write_byte(0x0004, 0x44);
    b.write_byte(0x0005, 0x45);
    b.write_byte(0x0006, 0x46);
    b.write_byte(0x0007, 0x47);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.b, 0x11);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.b, 0x15);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.b, 0x1f);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.b, 0x21);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.b, 0x25);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.b, 0x2f);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.b, 0x31);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.b, 0x3f);
    assert_eq!(c.reg.pc, 8);
}

#[test]
fn ld_c() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.b = 0x11;
    c.reg.c = 0x15;
    c.reg.d = 0x1F;
    c.reg.e = 0x21;
    c.reg.h = 0x25;
    c.reg.l = 0x2F;
    b.write_byte(0x252f, 0x31);
    c.reg.a = 0x3F;
    b.write_byte(0x0000, 0x48);
    b.write_byte(0x0001, 0x49);
    b.write_byte(0x0002, 0x4a);
    b.write_byte(0x0003, 0x4b);
    b.write_byte(0x0004, 0x4c);
    b.write_byte(0x0005, 0x4d);
    b.write_byte(0x0006, 0x4e);
    b.write_byte(0x0007, 0x4f);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.c, 0x11);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.c, 0x11);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.c, 0x1f);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.c, 0x21);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.c, 0x25);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.c, 0x2f);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.c, 0x31);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.c, 0x3f);
    assert_eq!(c.reg.pc, 8);
}

#[test]
fn ld_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.b = 0x11;
    c.reg.c = 0x15;
    c.reg.d = 0x1F;
    c.reg.e = 0x21;
    c.reg.h = 0x25;
    c.reg.l = 0x2F;
    b.write_byte(0x252f, 0x31);
    c.reg.a = 0x3F;
    b.write_byte(0x0000, 0x50);
    b.write_byte(0x0001, 0x51);
    b.write_byte(0x0002, 0x52);
    b.write_byte(0x0003, 0x53);
    b.write_byte(0x0004, 0x54);
    b.write_byte(0x0005, 0x55);
    b.write_byte(0x0006, 0x56);
    b.write_byte(0x0007, 0x57);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.d, 0x11);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.d, 0x15);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.d, 0x15);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.d, 0x21);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.d, 0x25);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.d, 0x2f);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.d, 0x31);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.d, 0x3f);
    assert_eq!(c.reg.pc, 8);
}

#[test]
fn ld_e() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.b = 0x11;
    c.reg.c = 0x15;
    c.reg.d = 0x1F;
    c.reg.e = 0x21;
    c.reg.h = 0x25;
    c.reg.l = 0x2F;
    b.write_byte(0x252f, 0x31);
    c.reg.a = 0x3F;
    b.write_byte(0x0000, 0x58);
    b.write_byte(0x0001, 0x59);
    b.write_byte(0x0002, 0x5a);
    b.write_byte(0x0003, 0x5b);
    b.write_byte(0x0004, 0x5c);
    b.write_byte(0x0005, 0x5d);
    b.write_byte(0x0006, 0x5e);
    b.write_byte(0x0007, 0x5f);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.e, 0x11);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.e, 0x15);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.e, 0x1f);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.e, 0x1f);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.e, 0x25);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.e, 0x2f);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.e, 0x31);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.e, 0x3f);
    assert_eq!(c.reg.pc, 8);
}

#[test]
fn ld_h() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.b = 0x11;
    c.reg.c = 0x15;
    c.reg.d = 0x1F;
    c.reg.e = 0x21;
    c.reg.h = 0x25;
    c.reg.l = 0x2F;
    b.write_byte(0x2f2f, 0x31);
    c.reg.a = 0x3F;
    b.write_byte(0x0000, 0x60);
    b.write_byte(0x0001, 0x61);
    b.write_byte(0x0002, 0x62);
    b.write_byte(0x0003, 0x63);
    b.write_byte(0x0004, 0x64);
    b.write_byte(0x0005, 0x65);
    b.write_byte(0x0006, 0x66);
    b.write_byte(0x0007, 0x67);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.h, 0x11);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.h, 0x15);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.h, 0x1f);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.h, 0x21);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.h, 0x21);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.h, 0x2f);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.h, 0x31);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.h, 0x3f);
    assert_eq!(c.reg.pc, 8);
}

#[test]
fn ld_l() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.b = 0x11;
    c.reg.c = 0x15;
    c.reg.d = 0x1F;
    c.reg.e = 0x21;
    c.reg.h = 0x25;
    c.reg.l = 0x2F;
    b.write_byte(0x2525, 0x31);
    c.reg.a = 0x3F;
    b.write_byte(0x0000, 0x68);
    b.write_byte(0x0001, 0x69);
    b.write_byte(0x0002, 0x6a);
    b.write_byte(0x0003, 0x6b);
    b.write_byte(0x0004, 0x6c);
    b.write_byte(0x0005, 0x6d);
    b.write_byte(0x0006, 0x6e);
    b.write_byte(0x0007, 0x6f);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.l, 0x11);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.l, 0x15);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.l, 0x1f);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.l, 0x21);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.l, 0x25);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.l, 0x25);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.l, 0x31);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.l, 0x3f);
    assert_eq!(c.reg.pc, 8);
}

#[test]
fn ld_hl_r() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.b = 0x11;
    c.reg.c = 0x15;
    c.reg.d = 0x1F;
    c.reg.e = 0x21;
    c.reg.h = 0x25;
    c.reg.l = 0x2F;
    b.write_byte(0x2f2f, 0x31);
    c.reg.a = 0x3F;
    b.write_byte(0x0000, 0x70);
    b.write_byte(0x0001, 0x71);
    b.write_byte(0x0002, 0x72);
    b.write_byte(0x0003, 0x73);
    b.write_byte(0x0004, 0x74);
    b.write_byte(0x0005, 0x75);
    b.write_byte(0x0006, 0x77);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(b.read_byte(0x252f), 0x11);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(b.read_byte(0x252f), 0x15);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(b.read_byte(0x252f), 0x1f);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(b.read_byte(0x252f), 0x21);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(b.read_byte(0x252f), 0x25);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(b.read_byte(0x252f), 0x2f);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(b.read_byte(0x252f), 0x3f);
    assert_eq!(c.reg.pc, 7);
}

#[test]
fn ld_a() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.b = 0x11;
    c.reg.c = 0x15;
    c.reg.d = 0x1F;
    c.reg.e = 0x21;
    c.reg.h = 0x25;
    c.reg.l = 0x2F;
    b.write_byte(0x252f, 0x31);
    c.reg.a = 0x3F;
    b.write_byte(0x0000, 0x78);
    b.write_byte(0x0001, 0x79);
    b.write_byte(0x0002, 0x7a);
    b.write_byte(0x0003, 0x7b);
    b.write_byte(0x0004, 0x7c);
    b.write_byte(0x0005, 0x7d);
    b.write_byte(0x0006, 0x7e);
    b.write_byte(0x0007, 0x7f);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.a, 0x11);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.a, 0x15);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.a, 0x1f);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.a, 0x21);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.a, 0x25);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.a, 0x2f);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.a, 0x31);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.a, 0x31);
    assert_eq!(c.reg.pc, 8);
}

#[test]
fn hlt() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x76);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0);
}

#[test]
fn ld_b_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.set_ix(0x25AF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x46);
    b.write_byte(0x0002, 0x19);
    b.write_byte(0x25C8, 0x39);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.b, 0x39);
    assert_eq!(c.reg.pc, 3);
}

#[test]
fn ld_b_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.set_iy(0x25AF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0x46);
    b.write_byte(0x0002, 0x19);
    b.write_byte(0x25C8, 0x39);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.b, 0x39);
    assert_eq!(c.reg.pc, 3);
}

#[test]
fn ld_ix_d_c() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.c = 0x1C;
    c.reg.set_ix(0x3100);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x71);
    b.write_byte(0x0002, 0x06);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(b.read_byte(0x3106), 0x1C);
    assert_eq!(c.reg.pc, 3);
}

#[test]
fn ld_ix_d_n() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.set_ix(0x219A);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x36);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x0003, 0x5A);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(b.read_byte(0x219F), 0x5A);
    assert_eq!(c.reg.pc, 4);
}

#[test]
fn ld_a_bc() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x0a);
    b.write_byte(0x100, 0x65);
    c.reg.set_bc(0x100);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0x65);
}

#[test]
fn ld_a_de() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x1a);
    b.write_byte(0x100, 0x65);
    c.reg.set_de(0x100);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0x65);
}

#[test]
fn ld_nn_a() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x32);
    b.write_byte(0x0001, 0x00);
    b.write_byte(0x0002, 0xff);
    c.reg.a = 0x56;
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(c.reg.pc, 0x0003);
    assert_eq!(b.read_byte(0xff00), 0x56);
}

#[test]
fn ld_a_r() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x5F);
    c.reg.r = 0x56;
    assert_eq!(c.execute(&mut b), 9);
    assert_eq!(c.reg.pc, 0x0002);
    assert_eq!(c.reg.a, 0x56);
}

#[test]
fn ld_dd_nn() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x21);
    b.write_byte(0x0001, 0x00);
    b.write_byte(0x0002, 0x50);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(c.reg.pc, 0x0003);
    assert_eq!(c.reg.get_hl(), 0x5000);
}

#[test]
fn ld_ix_nn() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x21);
    b.write_byte(0x0002, 0xA2);
    b.write_byte(0x0003, 0x45);
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(c.reg.get_ix(), 0x45A2);
}

#[test]
fn ld_hl_nn() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x2A);
    b.write_byte(0x0001, 0x45);
    b.write_byte(0x0002, 0x45);
    b.write_byte(0x4545, 0x37);
    b.write_byte(0x4546, 0xA1);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(c.reg.pc, 0x0003);
    assert_eq!(c.reg.get_hl(), 0xA137);
}

#[test]
fn ld_bc_cnn() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x4B);
    b.write_byte(0x0002, 0x30);
    b.write_byte(0x0003, 0x21);
    b.write_byte(0x2130, 0x65);
    b.write_byte(0x2131, 0x78);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(c.reg.get_bc(), 0x7865);
}

#[test]
fn ld_de_cnn() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x5B);
    b.write_byte(0x0002, 0x30);
    b.write_byte(0x0003, 0x21);
    b.write_byte(0x2130, 0x65);
    b.write_byte(0x2131, 0x78);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(c.reg.get_de(), 0x7865);
}

#[test]
fn ld_hl_cnn() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x6B);
    b.write_byte(0x0002, 0x30);
    b.write_byte(0x0003, 0x21);
    b.write_byte(0x2130, 0x65);
    b.write_byte(0x2131, 0x78);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(c.reg.get_hl(), 0x7865);
}

#[test]
fn ld_sp_cnn() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x7B);
    b.write_byte(0x0002, 0x30);
    b.write_byte(0x0003, 0x21);
    b.write_byte(0x2130, 0x65);
    b.write_byte(0x2131, 0x78);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(c.reg.sp, 0x7865);
}

#[test]
fn ld_ix_cnn() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x2A);
    b.write_byte(0x0002, 0x66);
    b.write_byte(0x0003, 0x66);
    b.write_byte(0x6666, 0x92);
    b.write_byte(0x6667, 0xDA);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(c.reg.get_ix(), 0xDA92);
}

#[test]
fn ld_iy_cnn() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0x2A);
    b.write_byte(0x0002, 0x66);
    b.write_byte(0x0003, 0x66);
    b.write_byte(0x6666, 0x92);
    b.write_byte(0x6667, 0xDA);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(c.reg.get_iy(), 0xDA92);
}

#[test]
fn ld_cnn_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x22);
    b.write_byte(0x0001, 0x29);
    b.write_byte(0x0002, 0xB2);
    c.reg.set_hl(0x483A);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(c.reg.pc, 0x0003);
    assert_eq!(b.read_byte(0xB229), 0x3A);
    assert_eq!(b.read_byte(0xB22A), 0x48);
}

#[test]
fn ld_ann_bc() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x43);
    b.write_byte(0x0002, 0x00);
    b.write_byte(0x0003, 0x10);
    c.reg.set_bc(0x4644);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(b.read_byte(0x1000), 0x44);
    assert_eq!(b.read_byte(0x1001), 0x46);
}

#[test]
fn ld_ann_de() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x53);
    b.write_byte(0x0002, 0x00);
    b.write_byte(0x0003, 0x10);
    c.reg.set_de(0x4644);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(b.read_byte(0x1000), 0x44);
    assert_eq!(b.read_byte(0x1001), 0x46);
}

#[test]
fn ld_ann_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x63);
    b.write_byte(0x0002, 0x00);
    b.write_byte(0x0003, 0x10);
    c.reg.set_hl(0x4644);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(b.read_byte(0x1000), 0x44);
    assert_eq!(b.read_byte(0x1001), 0x46);
}

#[test]
fn ld_ann_sp() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x73);
    b.write_byte(0x0002, 0x00);
    b.write_byte(0x0003, 0x10);
    c.reg.sp = 0x4644;
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(b.read_byte(0x1000), 0x44);
    assert_eq!(b.read_byte(0x1001), 0x46);
}

#[test]
fn ld_ann_ix() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x22);
    b.write_byte(0x0002, 0x38);
    b.write_byte(0x0003, 0x88);
    c.reg.set_ix(0x4174);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(b.read_byte(0x8838), 0x74);
    assert_eq!(b.read_byte(0x8839), 0x41);
}

#[test]
fn ld_ann_iy() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0x22);
    b.write_byte(0x0002, 0x38);
    b.write_byte(0x0003, 0x88);
    c.reg.set_iy(0x4174);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 0x0004);
    assert_eq!(b.read_byte(0x8838), 0x74);
    assert_eq!(b.read_byte(0x8839), 0x41);
}

#[test]
fn ld_sp_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xF9);
    c.reg.h = 0x50;
    c.reg.l = 0x6c;
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.sp, 0x506c)
}

#[test]
fn ld_sp_ix() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xF9);
    c.reg.set_ix(0x98DA);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.sp, 0x98DA)
}

#[test]
fn ld_sp_iy() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xF9);
    c.reg.set_iy(0x98DA);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.sp, 0x98DA)
}

#[test]
fn push_af() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xF5);
    c.reg.a = 0x22;
    c.reg.flags.set_from_byte(0x33);
    c.reg.sp = 0x1007;
    assert_eq!(c.flags(), 0b00110011);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.sp, 0x1005);
    assert_eq!(b.read_byte(0x1005), 0x33);
    assert_eq!(b.read_byte(0x1006), 0x22);
    assert_eq!(c.reg.sp, 0x1005);
}

#[test]
fn push_ix() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xE5);
    c.reg.set_ix(0x2233);
    c.reg.sp = 0x1007;
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(b.read_byte(0x1005), 0x33);
    assert_eq!(b.read_byte(0x1006), 0x22);
    assert_eq!(c.reg.sp, 0x1005);
}

#[test]
fn push_iy() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xE5);
    c.reg.set_iy(0x2233);
    c.reg.sp = 0x1007;
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(b.read_byte(0x1005), 0x33);
    assert_eq!(b.read_byte(0x1006), 0x22);
    assert_eq!(c.reg.sp, 0x1005);
}

#[test]
fn pop_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xE1);
    b.write_byte(0x1000, 0x55);
    b.write_byte(0x1001, 0x33);
    c.reg.sp = 0x1000;
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.get_hl(), 0x3355);
    assert_eq!(c.reg.sp, 0x1002);
}

#[test]
fn pop_ix() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xE1);
    b.write_byte(0x1000, 0x55);
    b.write_byte(0x1001, 0x33);
    c.reg.sp = 0x1000;
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_ix(), 0x3355);
    assert_eq!(c.reg.sp, 0x1002);
}

#[test]
fn pop_iy() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xE1);
    b.write_byte(0x1000, 0x55);
    b.write_byte(0x1001, 0x33);
    c.reg.sp = 0x1000;
    assert_eq!(c.execute(&mut b), 14);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_iy(), 0x3355);
    assert_eq!(c.reg.sp, 0x1002);
}

#[test]
fn ex_de_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xEB);
    c.reg.set_de(0x2822);
    c.reg.set_hl(0x499A);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.get_de(), 0x499A);
    assert_eq!(c.reg.get_hl(), 0x2822);
}

#[test]
fn ex_af_afp() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x08);
    c.reg.set_af(0x9900);
    assert_eq!(c.reg.get_af(), 0x9900);
    c.alt.set_af(0x5944);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.get_af(), 0x5944);
    assert_eq!(c.alt.get_af(), 0x9900);
}

#[test]
fn exx() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xD9);
    c.reg.set_bc(0x445A);
    c.reg.set_de(0x3DA2);
    c.reg.set_hl(0x8859);
    c.alt.set_bc(0x0988);
    c.alt.set_de(0x9300);
    c.alt.set_hl(0x00E7);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.get_bc(), 0x0988);
    assert_eq!(c.reg.get_de(), 0x9300);
    assert_eq!(c.reg.get_hl(), 0x00E7);
    assert_eq!(c.alt.get_bc(), 0x445A);
    assert_eq!(c.alt.get_de(), 0x3DA2);
    assert_eq!(c.alt.get_hl(), 0x8859);
}

#[test]
fn ex_sp_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xE3);
    c.reg.set_hl(0x7012);
    c.reg.sp = 0x8856;
    b.write_byte(0x8856, 0x11);
    b.write_byte(0x8857, 0x22);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.get_hl(), 0x2211);
    assert_eq!(b.read_byte(0x8856), 0x12);
    assert_eq!(b.read_byte(0x8857), 0x70);
    assert_eq!(c.reg.sp, 0x8856);
}

#[test]
fn ex_sp_ix() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xE3);
    c.reg.set_ix(0x3988);
    c.reg.sp = 0x0100;
    b.write_byte(0x0100, 0x90);
    b.write_byte(0x0101, 0x48);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_ix(), 0x4890);
    assert_eq!(b.read_byte(0x0100), 0x88);
    assert_eq!(b.read_byte(0x0101), 0x39);
    assert_eq!(c.reg.sp, 0x0100);
}

#[test]
fn ex_sp_iy() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xE3);
    c.reg.set_iy(0x3988);
    c.reg.sp = 0x0100;
    b.write_byte(0x0100, 0x90);
    b.write_byte(0x0101, 0x48);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_iy(), 0x4890);
    assert_eq!(b.read_byte(0x0100), 0x88);
    assert_eq!(b.read_byte(0x0101), 0x39);
    assert_eq!(c.reg.sp, 0x0100);
}

#[test]
fn ldi() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xA0);
    c.reg.set_hl(0x1111);
    c.reg.set_de(0x2222);
    c.reg.set_bc(0x07);
    b.write_byte(0x1111, 0x88);
    b.write_byte(0x2222, 0x66);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x1112);
    assert_eq!(b.read_byte(0x1111), 0x88);
    assert_eq!(c.reg.get_de(), 0x2223);
    assert_eq!(b.read_byte(0x2222), 0x88);
    assert_eq!(c.reg.get_bc(), 0x06);
}

#[test]
fn ldir() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB0);
    c.reg.set_hl(0x1111);
    c.reg.set_de(0x2222);
    c.reg.set_bc(0x0003);
    b.write_byte(0x1111, 0x88);
    b.write_byte(0x2222, 0x66);
    b.write_byte(0x1112, 0x36);
    b.write_byte(0x2223, 0x59);
    b.write_byte(0x1113, 0xA5);
    b.write_byte(0x2224, 0xC5);
    run_block(&mut c, &mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x1114);
    assert_eq!(b.read_byte(0x1111), 0x88);
    assert_eq!(b.read_byte(0x1112), 0x36);
    assert_eq!(b.read_byte(0x1113), 0xA5);
    assert_eq!(c.reg.get_de(), 0x2225);
    assert_eq!(b.read_byte(0x2222), 0x88);
    assert_eq!(b.read_byte(0x2223), 0x36);
    assert_eq!(b.read_byte(0x2224), 0xA5);
    assert_eq!(c.reg.get_bc(), 0x00);
}

#[test]
fn ldd() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xA8);
    c.reg.set_hl(0x1111);
    c.reg.set_de(0x2222);
    c.reg.set_bc(0x07);
    b.write_byte(0x1111, 0x88);
    b.write_byte(0x2222, 0x66);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x1110);
    assert_eq!(b.read_byte(0x1111), 0x88);
    assert_eq!(c.reg.get_de(), 0x2221);
    assert_eq!(b.read_byte(0x2222), 0x88);
    assert_eq!(c.reg.get_bc(), 0x06);
}

#[test]
fn lddr() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB8);
    c.reg.set_hl(0x1114);
    c.reg.set_de(0x2225);
    c.reg.set_bc(0x0003);
    b.write_byte(0x1112, 0x88);
    b.write_byte(0x2223, 0x66);
    b.write_byte(0x1113, 0x36);
    b.write_byte(0x2224, 0x59);
    b.write_byte(0x1114, 0xA5);
    b.write_byte(0x2225, 0xC5);
    run_block(&mut c, &mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x1111);
    assert_eq!(b.read_byte(0x1112), 0x88);
    assert_eq!(b.read_byte(0x1113), 0x36);
    assert_eq!(b.read_byte(0x1114), 0xA5);
    assert_eq!(c.reg.get_de(), 0x2222);
    assert_eq!(b.read_byte(0x2223), 0x88);
    assert_eq!(b.read_byte(0x2224), 0x36);
    assert_eq!(b.read_byte(0x2225), 0xA5);
    assert_eq!(c.reg.get_bc(), 0x00);
}

#[test]
fn cpi() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xA1);
    c.reg.a = 0x3B;
    c.reg.set_hl(0x1111);
    c.reg.set_bc(0x01);
    b.write_byte(0x1111, 0x3B);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x1112);
    assert_eq!(c.reg.get_bc(), 0);
    assert_eq!(c.reg.flags.z, true);
    assert_eq!(c.reg.flags.p, false);
}

#[test]
fn cpir() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB1);
    c.reg.a = 0xF3;
    c.reg.set_hl(0x1111);
    c.reg.set_bc(0x07);
    b.write_byte(0x1111, 0x52);
    b.write_byte(0x1112, 0x00);
    b.write_byte(0x1113, 0xF3);
    run_block(&mut c, &mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x1114);
    assert_eq!(c.reg.get_bc(), 4);
    assert_eq!(c.reg.flags.z, true);
    assert_eq!(c.reg.flags.p, true);
}

#[test]
fn cpd() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xA9);
    c.reg.a = 0x3B;
    c.reg.set_hl(0x1111);
    c.reg.set_bc(0x01);
    b.write_byte(0x1111, 0x3B);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x1110);
    assert_eq!(c.reg.get_bc(), 0);
    assert_eq!(c.reg.flags.z, true);
    assert_eq!(c.reg.flags.p, false);
}

#[test]
fn cpdr() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB9);
    c.reg.a = 0xF3;
    c.reg.set_hl(0x1118);
    c.reg.set_bc(0x07);
    b.write_byte(0x1116, 0xF3);
    b.write_byte(0x1117, 0x00);
    b.write_byte(0x1118, 0x52);
    run_block(&mut c, &mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x1115);
    assert_eq!(c.reg.get_bc(), 4);
    assert_eq!(c.reg.flags.z, true);
    assert_eq!(c.reg.flags.p, true);
}

#[test]
fn add_a_r() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x81);
    c.reg.a = 0x44;
    c.reg.c = 0x11;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0x55);
}

#[test]
fn add_a_n() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xC6);
    b.write_byte(0x0001, 0x33);
    c.reg.a = 0x23;
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.a, 0x56);
}

#[test]
fn add_a_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x86);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x22);
    c.reg.a = 0x11;
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.a, 0x33);
}

#[test]
fn add_a_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0x86);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x22);
    c.reg.a = 0x11;
    c.reg.set_iy(0x1000);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.a, 0x33);
}

#[test]
fn addc_a_r() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x8E);
    b.write_byte(0x6666, 0x10);
    c.reg.a = 0x16;
    c.reg.flags.c = true;
    c.reg.set_hl(0x6666);
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0x27);
}

#[test]
fn addc_a_n() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xCE);
    b.write_byte(0x0001, 0x10);
    c.reg.a = 0x16;
    c.reg.flags.c = true;
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.a, 0x27);
}

#[test]
fn sub_r() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x92);
    c.reg.a = 0x29;
    c.reg.d = 0x11;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0x18);
}

#[test]
fn sub_a_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x96);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x22);
    c.reg.a = 0x63;
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.a, 0x41);
}

#[test]
fn sub_a_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0x96);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x22);
    c.reg.a = 0x63;
    c.reg.set_iy(0x1000);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.a, 0x41);
}

#[test]
fn sbc_a_r() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x9E);
    b.write_byte(0x3433, 0x05);
    c.reg.a = 0x16;
    c.reg.set_hl(0x3433);
    c.reg.flags.c = true;
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0x10);
}

#[test]
fn sbc_a_r_ovf() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x9E);
    b.write_byte(0x3433, 0x01);
    c.reg.a = 0x80;
    c.reg.set_hl(0x3433);
    c.reg.flags.c = true;
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0x7E);
    assert_eq!(c.reg.flags.p, true);
}

#[test]
fn sbc_a_n() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDE);
    b.write_byte(0x0001, 0x05);
    c.reg.a = 0x16;
    c.reg.flags.c = true;
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.a, 0x10);
}

#[test]
fn sbc_a_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x9E);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x22);
    c.reg.a = 0x63;
    c.reg.flags.c = true;
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.a, 0x40);
}

#[test]
fn sbc_a_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0x9E);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x22);
    c.reg.a = 0x63;
    c.reg.flags.c = true;
    c.reg.set_iy(0x1000);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.a, 0x40);
}

#[test]
fn and_r() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xA0);
    c.reg.a = 0xC3;
    c.reg.b = 0x7B;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0x43);
}

#[test]
fn and_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xA6);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x7B);
    c.reg.a = 0xC3;
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.a, 0x43);
}

#[test]
fn and_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xA6);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x7B);
    c.reg.a = 0xC3;
    c.reg.set_iy(0x1000);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.a, 0x43);
}

#[test]
fn or_r() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xB4);
    c.reg.a = 0x12;
    c.reg.h = 0x48;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0x5A);
}

#[test]
fn or_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xB6);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x48);
    c.reg.a = 0x12;
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.a, 0x5A);
}

#[test]
fn or_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xB6);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x48);
    c.reg.a = 0x12;
    c.reg.set_iy(0x1000);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.a, 0x5A);
}

#[test]
fn xor_n() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xEE);
    b.write_byte(0x0001, 0x5D);
    c.reg.a = 0x96;
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.a, 0xCB);
}

#[test]
fn xor_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xAE);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x5D);
    c.reg.a = 0x96;
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.a, 0xCB);
}

#[test]
fn xor_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xAE);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x5D);
    c.reg.a = 0x96;
    c.reg.set_iy(0x1000);
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.a, 0xCB);
}

#[test]
fn cp_r() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xBB);
    c.reg.a = 0x0A;
    c.reg.e = 0x05;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.flags.z, false);
    assert_eq!(c.reg.flags.c, false);
}

#[test]
fn cp_n() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFE);
    b.write_byte(0x0001, 0x05);
    c.reg.a = 0x0A;
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.flags.z, false);
    assert_eq!(c.reg.flags.c, false);
}

#[test]
fn cp_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xBE);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x05);
    c.reg.a = 0x0A;
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.flags.z, false);
    assert_eq!(c.reg.flags.c, false);
}

#[test]
fn cp_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xBE);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x1005, 0x05);
    c.reg.a = 0x0A;
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 3);
    assert_eq!(c.reg.flags.z, false);
    assert_eq!(c.reg.flags.c, false);
}

#[test]
fn inc_b() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x04);
    c.reg.b = 0xff;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 0x0001);
    assert_eq!(0, c.reg.b);
    assert_eq!(true, c.reg.flags.z);
}

#[test]
fn inc_c() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x0C);
    c.reg.c = 0xff;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 0x0001);
    assert_eq!(0, c.reg.c);
    assert_eq!(true, c.reg.flags.z);
}

#[test]
fn inc_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x14);
    c.reg.d = 0xff;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 0x0001);
    assert_eq!(0, c.reg.d);
    assert_eq!(true, c.reg.flags.z);
}

#[test]
fn inc_e() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x1C);
    c.reg.e = 0xff;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 0x0001);
    assert_eq!(0, c.reg.e);
    assert_eq!(true, c.reg.flags.z);
}

#[test]
fn inc_h() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x24);
    c.reg.h = 0xff;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 0x0001);
    assert_eq!(0, c.reg.h);
    assert_eq!(true, c.reg.flags.z);
}

#[test]
fn inc_l() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x2C);
    c.reg.l = 0xff;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 0x0001);
    assert_eq!(0, c.reg.l);
    assert_eq!(true, c.reg.flags.z);
}

#[test]
fn inc_c_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x34);
    b.write_byte(0x0001, 0x34);
    b.write_byte(0x100, 0xff);
    c.reg.set_hl(0x100);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(c.reg.pc, 0x0001);
    assert_eq!(0, b.read_byte(0x100));
    assert_eq!(true, c.reg.flags.z);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x0002);
    assert_eq!(1, b.read_byte(0x100));
    assert_eq!(false, c.reg.flags.z);
}

#[test]
fn inc_a() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x3C);
    c.reg.a = 0x0f;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 0x0001);
    assert_eq!(0x10, c.reg.a);
    assert_eq!(false, c.reg.flags.z);
    assert_eq!(true, c.reg.flags.h);
}

#[test]
fn inc_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x34);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x105, 0xff);
    c.reg.set_ix(0x100);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 0x03);
    assert_eq!(0, b.read_byte(0x105));
    assert_eq!(true, c.reg.flags.z);
}

#[test]
fn inc_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0x34);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x105, 0xff);
    c.reg.set_iy(0x100);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 0x03);
    assert_eq!(0, b.read_byte(0x105));
    assert_eq!(true, c.reg.flags.z);
}

#[test]
fn dcr_b() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x05);
    b.write_byte(0x0001, 0x05);
    c.reg.b = 0x01;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(0, c.reg.b);
    assert_eq!(true, c.reg.flags.z);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(0xff, c.reg.b);
    assert_eq!(false, c.reg.flags.z);
}

#[test]
fn dcr_c() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x0d);
    b.write_byte(0x0001, 0x0d);
    c.reg.c = 0x01;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(0, c.reg.c);
    assert_eq!(true, c.reg.flags.z);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(0xff, c.reg.c);
    assert_eq!(false, c.reg.flags.z);
}

#[test]
fn dcr_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x15);
    b.write_byte(0x0001, 0x15);
    c.reg.d = 0x01;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(0, c.reg.d);
    assert_eq!(true, c.reg.flags.z);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(0xff, c.reg.d);
    assert_eq!(false, c.reg.flags.z);
}

#[test]
fn dcr_e() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x1d);
    b.write_byte(0x0001, 0x1d);
    c.reg.e = 0x01;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(0, c.reg.e);
    assert_eq!(true, c.reg.flags.z);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(0xff, c.reg.e);
    assert_eq!(false, c.reg.flags.z);
}

#[test]
fn dcr_h() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x25);
    b.write_byte(0x0001, 0x25);
    c.reg.h = 0x01;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(0, c.reg.h);
    assert_eq!(true, c.reg.flags.z);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(0xff, c.reg.h);
    assert_eq!(false, c.reg.flags.z);
}

#[test]
fn dcr_l() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x2d);
    b.write_byte(0x0001, 0x2d);
    c.reg.l = 0x01;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(0, c.reg.l);
    assert_eq!(true, c.reg.flags.z);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(0xff, c.reg.l);
    assert_eq!(false, c.reg.flags.z);
}

#[test]
fn dcr_m() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x35);
    b.write_byte(0x0001, 0x35);
    b.write_byte(0x100, 0x55);
    c.reg.set_hl(0x0100);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(0x54, b.read_byte(0x0100));
    assert_eq!(false, c.reg.flags.z);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(0x53, b.read_byte(0x0100));
    assert_eq!(false, c.reg.flags.z);
}

#[test]
fn dcr_a() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x3d);
    b.write_byte(0x0001, 0x3d);
    c.reg.a = 0x01;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(0, c.reg.a);
    assert_eq!(true, c.reg.flags.z);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(0xff, c.reg.a);
    assert_eq!(false, c.reg.flags.z);
}

#[test]
fn dec_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x35);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x105, 0xff);
    c.reg.set_ix(0x100);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 0x03);
    assert_eq!(0xFE, b.read_byte(0x105));
    assert_eq!(false, c.reg.flags.z);
}

#[test]
fn dec_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0x35);
    b.write_byte(0x0002, 0x05);
    b.write_byte(0x105, 0xff);
    c.reg.set_iy(0x100);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 0x03);
    assert_eq!(0xFE, b.read_byte(0x105));
    assert_eq!(false, c.reg.flags.z);
}

#[test]
fn daa() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x27);
    c.reg.a = 0x9B;
    c.reg.flags.h = false;
    c.reg.flags.c = false;
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 1);
    assert_eq!(c.reg.flags.h, true);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn neg_doc() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x44);
    c.reg.a = 0b10011000;
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(0b01101000, c.reg.a);
}

#[test]
fn neg_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/neg.bin", 0).unwrap();
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.a, 0x01); // LD A,0x01
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.a, 0xFF);
    assert_eq!(c.flags(), SF | HF | NF | CF); // NEG
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.a, 0x00);
    assert_eq!(c.flags(), ZF | HF | CF); // ADD A,0x01
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.a, 0x00);
    assert_eq!(c.flags(), ZF | NF); // NEG
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.a, 0x80);
    assert_eq!(c.flags(), SF | PF | NF | CF); // SUB A,0x80
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.a, 0x80);
    assert_eq!(c.flags(), SF | PF | NF | CF); // NEG
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.a, 0xC0);
    assert_eq!(c.flags(), SF); // ADD A,0x40
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.a, 0x40);
    assert_eq!(c.flags(), NF | CF); // NEG
}

#[test]
fn ccf() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x3f);
    b.write_byte(0x0001, 0x3f);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(true, c.reg.flags.c);
    assert_eq!(c.reg.pc, 0x0001);
    c.execute(&mut b);
    assert_eq!(false, c.reg.flags.c);
    assert_eq!(c.reg.pc, 0x0002);
}

#[test]
fn scf() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x37);
    b.write_byte(0x0001, 0x37);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 0x0001);
    assert_eq!(true, c.reg.flags.c);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x0002);
    assert_eq!(true, c.reg.flags.c);
}

#[test]
fn add_hl_b() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x09);
    c.reg.set_bc(0x339F);
    c.reg.set_hl(0xA17B);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(c.reg.h, 0xD5);
    assert_eq!(c.reg.l, 0x1A);
    assert_eq!(c.reg.flags.c, false);
    assert_eq!(c.reg.pc, 1);
}

#[test]
fn add_hl_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x19);
    c.reg.set_de(0x339F);
    c.reg.set_hl(0xA17B);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(c.reg.h, 0xD5);
    assert_eq!(c.reg.l, 0x1A);
    assert_eq!(c.reg.flags.c, false);
    assert_eq!(c.reg.pc, 1);
}

#[test]
fn add_hl_h() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x29);
    c.reg.set_hl(0x339F);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(c.reg.h, 0x67);
    assert_eq!(c.reg.l, 0x3e);
    assert_eq!(c.reg.flags.c, false);
    assert_eq!(c.reg.pc, 1);
}

#[test]
fn add_hl_sp() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x39);
    c.reg.sp = 0x339F;
    c.reg.set_hl(0xA17B);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(c.reg.h, 0xD5);
    assert_eq!(c.reg.l, 0x1A);
    assert_eq!(c.reg.flags.c, false);
    assert_eq!(c.reg.pc, 1);
}

#[test]
fn adc_hl_b() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x4A);
    c.reg.set_bc(0x2222);
    c.reg.set_hl(0x5437);
    c.reg.flags.c = true;
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(c.reg.h, 0x76);
    assert_eq!(c.reg.l, 0x5A);
    assert_eq!(c.reg.pc, 2);
}

#[test]
fn adc_hl_d_ovf() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x5A);
    c.reg.set_de(0x7FF0);
    c.reg.set_hl(0x000F);
    c.reg.flags.c = true;
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(c.reg.h, 0x80);
    assert_eq!(c.reg.l, 0x00);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.flags.p, true);
}

#[test]
fn adc_hl_h_ovf() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x6A);
    c.reg.set_hl(0x000F);
    c.reg.flags.c = true;
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(c.reg.h, 0x00);
    assert_eq!(c.reg.l, 0x1F);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.flags.p, false);
}

#[test]
fn adc_hl_sp_ovf() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x7A);
    c.reg.set_hl(0x7FF0);
    c.reg.sp = 0x000F;
    c.reg.flags.c = true;
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(c.reg.h, 0x80);
    assert_eq!(c.reg.l, 0x00);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.flags.p, true);
}

#[test]
fn sbc_hl_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x52);
    c.reg.set_hl(0x9999);
    c.reg.set_de(0x1111);
    c.reg.flags.c = true;
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(c.reg.h, 0x88);
    assert_eq!(c.reg.l, 0x87);
    assert_eq!(c.reg.pc, 2);
}

#[test]
fn add_ix_bc() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x09);
    c.reg.set_ix(0x3333);
    c.reg.set_bc(0x5555);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(c.reg.get_ix(), 0x8888);
    assert_eq!(c.reg.pc, 2);
}

#[test]
fn add_iy_bc() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0x09);
    c.reg.set_iy(0x3333);
    c.reg.set_bc(0x5555);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(c.reg.get_iy(), 0x8888);
    assert_eq!(c.reg.pc, 2);
}

#[test]
fn inc_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x23);
    c.reg.set_hl(0x1000);
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.get_hl(), 0x1001);
}

#[test]
fn inc_ix() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x23);
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_ix(), 0x1001);
}

#[test]
fn inc_iy() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0x23);
    c.reg.set_iy(0x1000);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_iy(), 0x1001);
}

#[test]
fn dec_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x2B);
    c.reg.set_hl(0x1001);
    assert_eq!(c.execute(&mut b), 6);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.get_hl(), 0x1000);
}

#[test]
fn dec_ix() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x2B);
    c.reg.set_ix(0x2006);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_ix(), 0x2005);
}

#[test]
fn dec_iy() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0x2B);
    c.reg.set_iy(0x2006);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_iy(), 0x2005);
}

#[test]
fn rlca() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x07);
    c.reg.a = 0b10001000;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0b00010001);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rla() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x17);
    c.reg.a = 0b01110110;
    c.reg.flags.c = true;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0b11101101);
    assert_eq!(c.reg.flags.c, false);
}

#[test]
fn rrca() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x0F);
    c.reg.a = 0b00010001;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0b10001000);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rra() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x1F);
    c.reg.a = 0b11100001;
    c.reg.flags.c = false;
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 1);
    assert_eq!(c.reg.a, 0b01110000);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rlc_a() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xCB);
    b.write_byte(0x0001, 0x07);
    c.reg.a = 0b10001000;
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.a, 0b00010001);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rlc_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xCB);
    b.write_byte(0x0001, 0x06);
    b.write_byte(0x2828, 0b10001000);
    c.reg.set_hl(0x2828);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(b.read_byte(0x2828), 0b00010001);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rlc_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x02);
    b.write_byte(0x0003, 0x06);
    b.write_byte(0x1002, 0b10001000);
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x1002), 0b00010001);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rlc_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x02);
    b.write_byte(0x0003, 0x06);
    b.write_byte(0x1002, 0b10001000);
    c.reg.set_iy(0x1000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x1002), 0b00010001);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rl_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xCB);
    b.write_byte(0x0001, 0x12);
    c.reg.d = 0b10001111;
    c.reg.flags.c = false;
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.d, 0b00011110);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rl_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x02);
    b.write_byte(0x0003, 0x16);
    b.write_byte(0x1002, 0b10001111);
    c.reg.flags.c = false;
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x1002), 0b00011110);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rl_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x02);
    b.write_byte(0x0003, 0x16);
    b.write_byte(0x1002, 0b10001111);
    c.reg.flags.c = false;
    c.reg.set_iy(0x1000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x1002), 0b00011110);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn read_le_dword() {
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x02);
    b.write_byte(0x0003, 0x16);
    assert_eq!(b.read_le_dword(0), 0xFDCB0216);
}

#[test]
fn rrc_a() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xCB);
    b.write_byte(0x0001, 0x0F);
    c.reg.a = 0b00110001;
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.a, 0b10011000);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rrc_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x02);
    b.write_byte(0x0003, 0x0E);
    b.write_byte(0x1002, 0b00110001);
    c.reg.flags.c = false;
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x1002), 0b10011000);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rrc_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x02);
    b.write_byte(0x0003, 0x0E);
    b.write_byte(0x1002, 0b00110001);
    c.reg.flags.c = false;
    c.reg.set_iy(0x1000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x1002), 0b10011000);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rr_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xCB);
    b.write_byte(0x0001, 0x1E);
    b.write_byte(0x4343, 0b11011101);
    c.reg.set_hl(0x4343);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(b.read_byte(0x4343), 0b01101110);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rr_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x02);
    b.write_byte(0x0003, 0x1E);
    b.write_byte(0x1002, 0b11011101);
    c.reg.flags.c = false;
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x1002), 0b01101110);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rr_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x02);
    b.write_byte(0x0003, 0x1E);
    b.write_byte(0x1002, 0b11011101);
    c.reg.flags.c = false;
    c.reg.set_iy(0x1000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x1002), 0b01101110);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn sla_l() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xCB);
    b.write_byte(0x0001, 0x25);
    c.reg.l = 0b10110001;
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.l, 0b01100010);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn sla_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x02);
    b.write_byte(0x0003, 0x26);
    b.write_byte(0x1002, 0b10110001);
    c.reg.flags.c = false;
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x1002), 0b01100010);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn sla_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x02);
    b.write_byte(0x0003, 0x26);
    b.write_byte(0x1002, 0b10110001);
    c.reg.flags.c = false;
    c.reg.set_iy(0x1000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x1002), 0b01100010);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn sra_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x02);
    b.write_byte(0x0003, 0x2E);
    b.write_byte(0x1002, 0b10111000);
    c.reg.flags.c = false;
    c.reg.set_ix(0x1000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x1002), 0b11011100);
    assert_eq!(c.reg.flags.c, false);
}

#[test]
fn srl_b() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xCB);
    b.write_byte(0x0001, 0x38);
    c.reg.b = 0b10001111;
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.b, 0b01000111);
    assert_eq!(c.reg.flags.c, true);
}

#[test]
fn rld() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x6F);
    b.write_byte(0x5000, 0b00110001);
    c.reg.set_hl(0x5000);
    c.reg.a = 0b01111010;
    assert_eq!(c.execute(&mut b), 18);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.a, 0b01110011);
    assert_eq!(b.read_byte(0x5000), 0b00011010);
}

#[test]
fn rrd() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0x67);
    b.write_byte(0x5000, 0b00100000);
    c.reg.set_hl(0x5000);
    c.reg.a = 0b10000100;
    assert_eq!(c.execute(&mut b), 18);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.a, 0b10000000);
    assert_eq!(b.read_byte(0x5000), 0b01000010);
}

#[test]
fn bit_4_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xCB);
    b.write_byte(0x0001, 0x66);
    b.write_byte(0x4444, 0x10);
    c.reg.set_hl(0x4444);
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.flags.z, false);
    assert_eq!(b.read_byte(0x4444), 0x10);
}

#[test]
fn bit_6_ix_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x04);
    b.write_byte(0x0003, 0x76);
    b.write_byte(0x2004, 0x40);
    c.reg.set_ix(0x2000);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x2004), 0x40);
    assert_eq!(c.reg.flags.z, false);
}

#[test]
fn bit_6_iy_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x04);
    b.write_byte(0x0003, 0x76);
    b.write_byte(0x2004, 0x40);
    c.reg.set_iy(0x2000);
    assert_eq!(c.execute(&mut b), 20);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x2004), 0x40);
    assert_eq!(c.reg.flags.z, false);
}

#[test]
fn set_4_a() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xCB);
    b.write_byte(0x0001, 0xE7);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.a, 0x10);
}

#[test]
fn set_4_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xCB);
    b.write_byte(0x0001, 0xE6);
    c.reg.set_hl(0x4444);
    assert_eq!(c.execute(&mut b), 15);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(b.read_byte(0x4444), 0x10);
}

#[test]
fn set_0_ix() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x03);
    b.write_byte(0x0003, 0xC6);
    c.reg.set_ix(0x2000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x2003), 0x01);
}

#[test]
fn set_0_iy() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x03);
    b.write_byte(0x0003, 0xC6);
    c.reg.set_iy(0x2000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x2003), 0x01);
}

#[test]
fn res_6_d() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xCB);
    b.write_byte(0x0001, 0xB2);
    c.reg.d = 0xFF;
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.d, 0xBF);
}

#[test]
fn reset_0_ix() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x03);
    b.write_byte(0x0003, 0xB6);
    b.write_byte(0x2003, 0xFF);
    c.reg.set_ix(0x2000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x2003), 0xBF);
}

#[test]
fn reset_0_iy() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFD);
    b.write_byte(0x0001, 0xCB);
    b.write_byte(0x0002, 0x03);
    b.write_byte(0x0003, 0xB6);
    b.write_byte(0x2003, 0xFF);
    c.reg.set_iy(0x2000);
    assert_eq!(c.execute(&mut b), 23);
    assert_eq!(c.reg.pc, 4);
    assert_eq!(b.read_byte(0x2003), 0xBF);
}

#[test]
fn jp() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xC3);
    b.write_byte(0x0001, 0x00);
    b.write_byte(0x0002, 0x3E);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(c.reg.pc, 0x3e00);
}

#[test]
fn jr() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0480;
    b.write_byte(0x0480, 0x18);
    b.write_byte(0x0481, 0x03);
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(c.reg.pc, 0x0485);
}

#[test]
fn jr_neg() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0480;
    b.write_byte(0x0480, 0x18);
    b.write_byte(0x0481, 0xFA);
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(c.reg.pc, 0x047C);
}

#[test]
fn jr_c_e() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0480;
    b.write_byte(0x0480, 0x38);
    b.write_byte(0x0481, 0xFA);
    c.reg.flags.c = true;
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(c.reg.pc, 0x047C);
}

#[test]
fn jr_nc_e() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0480;
    b.write_byte(0x0480, 0x30);
    b.write_byte(0x0481, 0xFA);
    c.reg.flags.c = false;
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(c.reg.pc, 0x047C);
}

#[test]
fn jr_z_e() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0300;
    b.write_byte(0x0300, 0x28);
    b.write_byte(0x0301, 0x03);
    c.reg.flags.z = true;
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(c.reg.pc, 0x0305);
}

#[test]
fn jr_nz_e() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0480;
    b.write_byte(0x0480, 0x20);
    b.write_byte(0x0481, 0xFA);
    c.reg.flags.z = false;
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(c.reg.pc, 0x047C);
}

#[test]
fn jp_hl() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x1000;
    b.write_byte(0x1000, 0xE9);
    c.reg.set_hl(0x4800);
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 0x4800);
}

#[test]
fn jp_ix() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x1000;
    b.write_byte(0x1000, 0xDD);
    b.write_byte(0x1001, 0xE9);
    c.reg.set_ix(0x4800);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.pc, 0x4800);
}

#[test]
fn jp_iy() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x1000;
    b.write_byte(0x1000, 0xFD);
    b.write_byte(0x1001, 0xE9);
    c.reg.set_iy(0x4800);
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.pc, 0x4800);
}

#[test]
fn call_nn() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x1A47;
    c.reg.sp = 0x3002;
    b.write_byte(0x1A47, 0xCD);
    b.write_byte(0x1A48, 0x35);
    b.write_byte(0x1A49, 0x21);
    assert_eq!(c.execute(&mut b), 17);
    assert_eq!(b.read_byte(0x3001), 0x1A);
    assert_eq!(b.read_byte(0x3000), 0x4A);
    assert_eq!(c.reg.sp, 0x3000);
    assert_eq!(c.reg.pc, 0x2135);
}

#[test]
fn call_cc_nn() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.flags.c = false;
    c.reg.pc = 0x1A47;
    c.reg.sp = 0x3002;
    b.write_byte(0x1A47, 0xD4);
    b.write_byte(0x1A48, 0x35);
    b.write_byte(0x1A49, 0x21);
    assert_eq!(c.execute(&mut b), 17);
    assert_eq!(b.read_byte(0x3001), 0x1A);
    assert_eq!(b.read_byte(0x3000), 0x4A);
    assert_eq!(c.reg.sp, 0x3000);
    assert_eq!(c.reg.pc, 0x2135);
}

#[test]
fn ret() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x3535;
    c.reg.sp = 0x2000;
    b.write_byte(0x3535, 0xC9);
    b.write_byte(0x2000, 0xB5);
    b.write_byte(0x2001, 0x18);
    assert_eq!(c.execute(&mut b), 10);
    assert_eq!(c.reg.sp, 0x2002);
    assert_eq!(c.reg.pc, 0x18B5);
}

#[test]
fn ret_cc() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.flags.s = true;
    c.reg.pc = 0x3535;
    c.reg.sp = 0x2000;
    b.write_byte(0x3535, 0xF8);
    b.write_byte(0x2000, 0xB5);
    b.write_byte(0x2001, 0x18);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(c.reg.sp, 0x2002);
    assert_eq!(c.reg.pc, 0x18B5);
}

#[test]
fn rst() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x15B3;
    b.write_byte(0x15B3, 0xDF);
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(c.reg.pc, 0x0018);
}

/// Un préfixe d'index devant une instruction qui n'utilise ni HL ni (HL) est
/// sans objet : le Z80 le traverse en quatre cycles, puis exécute
/// l'instruction telle quelle. Elle ne doit donc rien coûter de plus.
#[test]
fn an_ineffective_index_prefix_is_just_traversed() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0x00); // NOP

    assert_eq!(c.execute(&mut b), 4, "le prefixe seul coute 4 cycles");
    assert_eq!(c.reg.pc, 0x0001);
    assert_eq!(c.execute(&mut b), 4, "puis le NOP, inchange");
    assert_eq!(c.reg.pc, 0x0002);
    assert_eq!(
        c.unimplemented_count(),
        0,
        "ce n'est pas une instruction absente"
    );
}

/// Quand les préfixes s'enchaînent, c'est le dernier qui compte : DD FD 23
/// incrémente IY, pas IX.
#[test]
fn the_last_index_prefix_wins() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xDD);
    b.write_byte(0x0001, 0xFD);
    b.write_byte(0x0002, 0x23); // INC
    c.reg.set_ix(0x1111);
    c.reg.set_iy(0x2222);

    let mut cycles = c.execute(&mut b);
    cycles += c.execute(&mut b);

    assert_eq!(c.reg.get_iy(), 0x2223, "IY doit avoir ete incremente");
    assert_eq!(c.reg.get_ix(), 0x1111, "IX doit etre intact");
    assert_eq!(cycles, 14, "4 cycles de prefixe ignore, puis INC IY");
}

/// Les trous de la table ED ne sont pas des instructions absentes : le
/// processeur les traverse sans rien faire, en huit cycles et deux octets.
#[test]
fn the_holes_of_the_ed_table_do_nothing() {
    for op in [0x00u8, 0x3F, 0x77, 0x7F, 0xC0, 0xFF] {
        let mut c = CPU::new();
        let mut b = FlatBus::new(0xFFFF);
        b.write_byte(0x0000, 0xED);
        b.write_byte(0x0001, op);
        let a_before = c.reg.a;

        assert_eq!(c.execute(&mut b), 8, "ED {op:02X}");
        assert_eq!(c.reg.pc, 0x0002, "ED {op:02X}");
        assert_eq!(c.flags(), 0, "ED {op:02X} ne doit toucher aucun drapeau");
        assert_eq!(
            c.reg.a, a_before,
            "ED {op:02X} ne doit toucher aucun registre"
        );
        assert_eq!(c.unimplemented_count(), 0, "ED {op:02X}");
    }
}

/// Le filet de sécurité : aucun opcode ne doit rester non géré, et aucun ne
/// doit inventer sa durée. Une instruction qui renvoyait 255 cycles injectait
/// une scanline entière de temps émulé — de quoi désaccorder la vidéo, les
/// interruptions et le son sans le moindre message.
#[test]
fn no_opcode_is_left_unimplemented() {
    let mut sequences: Vec<[u8; 4]> = Vec::new();
    for op in 0..=0xFFu8 {
        sequences.push([op, 0x00, 0x00, 0x00]);
    }
    for prefix in [0xCBu8, 0xDD, 0xED, 0xFD] {
        for op in 0..=0xFFu8 {
            sequences.push([prefix, op, 0x02, 0x03]);
        }
    }
    for prefix in [0xDDu8, 0xFD] {
        for op in 0..=0xFFu8 {
            sequences.push([prefix, 0xCB, 0x02, op]);
        }
    }

    for bytes in sequences {
        let mut c = CPU::new();
        let mut b = FlatBus::new(0xFFFF);
        for (i, x) in bytes.iter().enumerate() {
            b.write_byte(i as u16, *x);
        }

        let cycles = c.execute(&mut b);

        assert!(
            c.take_unimplemented().is_none(),
            "{bytes:02X?} : {}",
            crate::dasm::dasm(&b, 0).0
        );
        assert!(
            (4..=23).contains(&cycles),
            "{bytes:02X?} : {cycles} cycles, {}",
            crate::dasm::dasm(&b, 0).0
        );
    }
}

/// Et le mécanisme de relevé lui-même : il doit rendre l'instruction fautive
/// une seule fois, sans jamais perdre le compte.
#[test]
fn an_unimplemented_instruction_is_reported_once_and_counted() {
    let mut c = CPU::new();
    assert!(c.take_unimplemented().is_none());
    assert_eq!(c.unimplemented_count(), 0);

    // Toutes les instructions du Z80 etant desormais gerees, on eprouve le
    // mecanisme sur le seul cas qui reste possible : aucune. Le contrat porte
    // donc sur son etat de repos, verifie apres un balayage complet.
    let mut b = FlatBus::new(0xFFFF);
    for op in 0..=0xFFu8 {
        b.write_byte(0x0000, op);
        b.write_byte(0x0001, 0x02);
        b.write_byte(0x0002, 0x03);
        b.write_byte(0x0003, 0x04);
        c.reg.pc = 0;
        c.execute(&mut b);
    }
    assert!(c.take_unimplemented().is_none());
    assert_eq!(c.unimplemented_count(), 0);
}

// if this test loops forever, interrupts are not working
#[test]
fn int() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFB); // EI
    b.write_byte(0x0001, 0x00); // NOP (must execute before IRQ)
    b.write_byte(0x0002, 0x00); // NOP
    c.reg.sp = 0x2000;
    c.execute(&mut b);
    c.int_request(0xCF);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x0002);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x0008);
    assert_eq!(c.reg.sp, 0x1FFE);
    assert_eq!(b.read_word(0x1FFE), 0x0002);
}

#[test]
fn int_im1() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED); // IM 1
    b.write_byte(0x0001, 0x56);
    b.write_byte(0x0002, 0xFB); // EI
    b.write_byte(0x0003, 0x00); // NOP (must execute before IRQ)
    b.write_byte(0x0004, 0x00); // NOP
    c.reg.sp = 0x2000;
    c.execute(&mut b);
    c.execute(&mut b);
    c.int_request(0xDF);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x0004);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x0038);
    assert_eq!(c.reg.sp, 0x1FFE);
    assert_eq!(b.read_word(0x1FFE), 0x0004);
}

#[test]
fn int_im2() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0x3E); // LD A,0x01
    b.write_byte(0x0001, 0x01);
    b.write_byte(0x0002, 0xED); // LD I,A
    b.write_byte(0x0003, 0x47);
    b.write_byte(0x0004, 0xED); // IM 2
    b.write_byte(0x0005, 0x5E);
    b.write_byte(0x0006, 0xFB); // EI
    b.write_byte(0x0007, 0x00); // NOP (must execute before IRQ)
    b.write_byte(0x0008, 0x00); // NOP
    b.write_word(0x0102, 0x1234);
    c.reg.sp = 0x2000;
    c.execute(&mut b);
    c.execute(&mut b);
    c.execute(&mut b);
    c.execute(&mut b);
    c.int_request(0x02);
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x0008);
    // IM 2 acknowledge: 19 T-states, and nothing else is executed during that call.
    assert_eq!(c.execute(&mut b), 19);
    assert_eq!(c.reg.pc, 0x1234);
    assert_eq!(c.reg.sp, 0x1FFE);
    assert_eq!(b.read_word(0x1FFE), 0x0008);
    // The first instruction of the service routine (a NOP here) runs on the next call.
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 0x1235);
}

#[test]
fn nmi() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFB); // EI
    b.write_byte(0x0066, 0x47); // LD B,A
    c.reg.a = 0x0F;
    c.reg.sp = 0x2000;
    c.execute(&mut b);
    c.nmi_request();
    // NMI acknowledge: 11 T-states, and nothing else is executed during that call.
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(c.reg.pc, 0x0066);
    assert_eq!(c.reg.sp, 0x1FFE);
    assert_eq!(b.read_word(0x1FFE), 0x0001);
    // The first instruction of the handler runs on the next call.
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.pc, 0x0067);
    assert_eq!(c.reg.b, 0x0F);
}

// RETN: verifies that returning from NMI restores IFF1 from IFF2, re-enabling maskable interrupts.
#[test]
fn retn() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFB); // EI
    // NMI handler at 0x0066: RETN
    b.write_byte(0x0066, 0xED);
    b.write_byte(0x0067, 0x45); // RETN
    c.reg.sp = 0x2000;
    c.execute(&mut b); // EI: IFF1=true, IFF2=true
    c.nmi_request();
    // NMI fires: IFF2=true (saved), IFF1=false, PC jumps to 0x0066
    c.execute(&mut b);
    // RETN executes at 0x0066: IFF1=IFF2=true, returns to 0x0001
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x0001); // returned from NMI handler
    assert_eq!(c.reg.sp, 0x2000); // stack fully restored
    // Verify IFF1=true: a maskable interrupt must now fire
    c.int_request(0xFF);
    c.execute(&mut b); // INT fires (RST 38h) because IFF1 was restored to true by RETN
    assert_eq!(c.reg.pc, 0x0038);
}

// RETI: basic return-from-interrupt test (IM 1). Verifies that RETI pops the correct return
// address and fully restores the stack pointer.
#[test]
fn reti() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED); // IM 1
    b.write_byte(0x0001, 0x56);
    b.write_byte(0x0002, 0xFB); // EI
    b.write_byte(0x0003, 0x00); // NOP (must execute before IRQ)
    b.write_byte(0x0004, 0x00); // NOP (INT fires at the start of this step)
    // ISR at 0x0038: NOP then RETI
    b.write_byte(0x0038, 0x00); // NOP
    b.write_byte(0x0039, 0xED);
    b.write_byte(0x003A, 0x4D); // RETI
    c.reg.sp = 0x2000;
    c.execute(&mut b); // IM 1
    c.execute(&mut b); // EI
    c.int_request(0xDF);
    c.execute(&mut b); // NOP at 0x0003: delay expires, INT not yet taken
    assert_eq!(c.reg.pc, 0x0004);
    c.execute(&mut b); // INT fires: RST 38h → PC=0x0038, pushes 0x0004
    assert_eq!(c.reg.pc, 0x0038);
    assert_eq!(c.reg.sp, 0x1FFE);
    assert_eq!(b.read_word(0x1FFE), 0x0004);
    c.execute(&mut b); // NOP at 0x0038
    c.execute(&mut b); // RETI: restores PC and SP
    assert_eq!(c.reg.pc, 0x0004); // returned to the instruction that was interrupted
    assert_eq!(c.reg.sp, 0x2000); // stack fully restored
    c.execute(&mut b); // NOP at 0x0004: execution continues normally in main
    assert_eq!(c.reg.pc, 0x0005);
}

// RETI must copy IFF2 to IFF1, just like RETN. This test sets IFF2=true via EI and then triggers
// an NMI (which sets IFF1=false, IFF2=true). The NMI handler uses RETI instead of the canonical
// RETN to prove that RETI also restores IFF1 from IFF2 as required by the Z80 specification.
// In normal Z80 code RETN is used after NMI and RETI after maskable interrupts; this non-standard
// combination is the only straightforward way to create a state where IFF2 != IFF1 before RETI
// and thus distinguish a correct implementation from a buggy one.
#[test]
fn reti_iff_restore() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xFB); // EI
    // NMI handler at 0x0066: RETI (instead of the canonical RETN — intentional for this test)
    b.write_byte(0x0066, 0xED);
    b.write_byte(0x0067, 0x4D); // RETI
    c.reg.sp = 0x2000;
    c.execute(&mut b); // EI: IFF1=true, IFF2=true
    c.nmi_request();
    // NMI fires: IFF2=true (saved from IFF1), IFF1=false, PC jumps to 0x0066
    c.execute(&mut b);
    // RETI executes at 0x0066: must set IFF1=IFF2=true, then return to 0x0001
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x0001);
    assert_eq!(c.reg.sp, 0x2000);
    // Verify RETI restored IFF1=IFF2=true: a maskable interrupt must now be accepted
    c.int_request(0xFF);
    c.execute(&mut b); // INT fires because IFF1 was correctly restored to true by RETI
    assert_eq!(c.reg.pc, 0x0038);
    assert_eq!(c.reg.sp, 0x1FFE);
}

// Nesting of interrupts: two nested IM 1 maskable interrupts are serviced one inside the other.
// ISR at 0x0038 calls EI to re-enable interrupts, allowing a second INT to fire while the first
// ISR is still active. Each ISR ends with RETI. The test checks that return addresses are correct
// at every level and that the stack is fully restored after both RETI instructions.
#[test]
fn reti_nesting() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    // Main: IM 1 then EI (NOPs follow by default)
    b.write_byte(0x0000, 0xED); // IM 1
    b.write_byte(0x0001, 0x56);
    b.write_byte(0x0002, 0xFB); // EI
    // ISR at 0x0038 (shared by ISR1 and ISR2): EI, NOP, NOP, RETI
    b.write_byte(0x0038, 0xFB); // EI  (re-enables interrupts, allowing nesting)
    b.write_byte(0x0039, 0x00); // NOP
    b.write_byte(0x003A, 0x00); // NOP (INT2 fires here when executing ISR1)
    b.write_byte(0x003B, 0xED);
    b.write_byte(0x003C, 0x4D); // RETI
    c.reg.sp = 0x8000;
    c.execute(&mut b); // IM 1
    c.execute(&mut b); // EI
    // INT1: fires after EI delay expires
    c.int_request(0xFF);
    c.execute(&mut b); // NOP: EI delay expires
    c.execute(&mut b); // INT1 fires → RST 38h, pushes return address, enters ISR1
    assert_eq!(c.reg.pc, 0x0038);
    assert_eq!(c.reg.sp, 0x7FFE);
    assert_eq!(b.read_word(0x7FFE), 0x0004); // return address: main NOP that was about to run
    // ISR1: EI re-enables interrupts
    c.execute(&mut b); // EI in ISR1
    // INT2: fires while inside ISR1 (nesting)
    c.int_request(0xFF);
    c.execute(&mut b); // NOP: EI delay expires
    c.execute(&mut b); // INT2 fires → RST 38h, pushes return address, enters ISR2
    assert_eq!(c.reg.pc, 0x0038);
    assert_eq!(c.reg.sp, 0x7FFC);
    assert_eq!(b.read_word(0x7FFC), 0x003A); // return address: NOP in ISR1 that was about to run
    // ISR2: EI, NOPs, then RETI (no third interrupt is requested)
    c.execute(&mut b); // EI in ISR2
    c.execute(&mut b); // NOP
    c.execute(&mut b); // NOP
    c.execute(&mut b); // RETI from ISR2: returns to ISR1 (0x003A)
    assert_eq!(c.reg.pc, 0x003A); // back in ISR1
    assert_eq!(c.reg.sp, 0x7FFE); // one stack frame removed
    // ISR1 resumes from 0x003A and returns
    c.execute(&mut b); // NOP at 0x003A
    c.execute(&mut b); // RETI from ISR1: returns to main (0x0004)
    assert_eq!(c.reg.pc, 0x0004); // back in main
    assert_eq!(c.reg.sp, 0x8000); // stack fully restored
    // After both ISRs used EI+RETI, IFF1 must be true: a new interrupt must fire
    c.int_request(0xFF);
    c.execute(&mut b); // INT fires immediately because IFF1 was correctly restored
    assert_eq!(c.reg.pc, 0x0038);
}

#[test]
fn jr_nz_neg() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0274;
    b.write_byte(0x0274, 0x0E); // LD C,$08
    b.write_byte(0x0275, 0x08);
    b.write_byte(0x0276, 0x0D); // DEC C
    b.write_byte(0x0277, 0x20); // JR NZ,$F2
    b.write_byte(0x0278, 0xF2);
    for _ in 0..3 {
        c.execute(&mut b);
    }
    assert_eq!(c.reg.pc, 0x026B);
}

#[test]
fn jr_nz_neg_false() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0274;
    b.write_byte(0x0274, 0x0E); // LD C,$08
    b.write_byte(0x0275, 0x01);
    b.write_byte(0x0276, 0x0D); // DEC C
    b.write_byte(0x0277, 0x20); // JR NZ,$F2
    b.write_byte(0x0278, 0xF2);
    for _ in 0..3 {
        c.execute(&mut b);
    }
    assert_eq!(c.reg.pc, 0x0279);
}

#[test]
fn dasm_cb() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0274;
    b.write_byte(0x0274, 0xCB); // RLC B
    b.write_byte(0x0275, 0x00);
    b.write_byte(0x0276, 0xCB); // BIT 1,B
    b.write_byte(0x0277, 0x48);
    assert_eq!(
        crate::dasm::dasm(&b, 0x274),
        (String::from("CB00          RLC B"), 2)
    );
    assert_eq!(
        crate::dasm::dasm(&b, 0x276),
        (String::from("CB48          BIT 1,B"), 2)
    );
}

#[test]
fn ldir_bc_zero() {
    // BC = 0 avant exécution : le compteur passe à 0xFFFF au premier
    // décrément, donc l'instruction se répète (65536 itérations au total).
    //
    // On vérifie ici le point essentiel : après une itération, l'instruction
    // n'est PAS terminée et le PC est resté sur son opcode pour la rejouer.
    // C'est ce qui la rend interruptible. Le décompte complet des 65536
    // itérations est couvert par ldir_with_zero_bc_repeats_64kb, qui copie la
    // mémoire sur elle-même : ici le balayage finirait par écraser l'opcode,
    // et un vrai Z80, qui le relit à chaque itération, exécuterait alors
    // n'importe quoi.
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB0);
    c.reg.set_hl(0x1000);
    c.reg.set_de(0x2000);
    c.reg.set_bc(0x0000);
    b.write_byte(0x1000, 0xAB);

    let cycles = c.execute(&mut b);

    assert_eq!(c.reg.get_bc(), 0xFFFF, "BC doit reboucler a 0xFFFF");
    assert_eq!(c.reg.pc, 0x0000, "l'instruction doit se rejouer");
    assert_eq!(cycles, 21, "une iteration qui se repete coute 21 cycles");
    assert_eq!(b.read_byte(0x2000), 0xAB, "le premier octet est copie");
    assert_eq!(c.flags() & (HF | NF), 0);
}

// CPI with no match: Z=0, S set when result is negative, H set on nibble borrow, N=1, C unchanged
#[test]
fn cpi_no_match() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xA1);
    // A=0x01, (HL)=0x02 → result = 0xFF (negative), H=1 (nibble borrow: 0x1 < 0x2), Z=0
    c.reg.a = 0x01;
    c.reg.set_hl(0x2000);
    c.reg.set_bc(0x03);
    c.reg.flags.c = true; // C must be preserved
    b.write_byte(0x2000, 0x02);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x2001);
    assert_eq!(c.reg.get_bc(), 0x02);
    assert_eq!(c.reg.flags.z, false); // no match
    assert_eq!(c.reg.flags.s, true); // result 0xFF has bit 7 set
    assert_eq!(c.reg.flags.h, true); // borrow from bit 4 (0x1 < 0x2)
    assert_eq!(c.reg.flags.n, true); // subtraction
    assert_eq!(c.reg.flags.p, true); // BC=2 after decrement ≠ 0
    assert_eq!(c.reg.flags.c, true); // C must be unchanged
}

// CPI: no half-carry when no nibble borrow
#[test]
fn cpi_no_half_carry() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xA1);
    // A=0x20, (HL)=0x10 → result=0x10 (positive), H=0 (0x0 >= 0x0, no borrow)
    c.reg.a = 0x20;
    c.reg.set_hl(0x2000);
    c.reg.set_bc(0x02);
    b.write_byte(0x2000, 0x10);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(c.reg.flags.z, false);
    assert_eq!(c.reg.flags.s, false); // result 0x10 is positive
    assert_eq!(c.reg.flags.h, false); // 0x0 >= 0x0, no borrow
    assert_eq!(c.reg.flags.n, true);
}

// CPD with no match: same flag rules as CPI but HL decremented
#[test]
fn cpd_no_match() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xA9);
    c.reg.a = 0x01;
    c.reg.set_hl(0x2000);
    c.reg.set_bc(0x03);
    c.reg.flags.c = false; // C must be preserved
    b.write_byte(0x2000, 0x02);
    assert_eq!(c.execute(&mut b), 16);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x1FFF); // decremented
    assert_eq!(c.reg.get_bc(), 0x02);
    assert_eq!(c.reg.flags.z, false);
    assert_eq!(c.reg.flags.s, true);
    assert_eq!(c.reg.flags.h, true);
    assert_eq!(c.reg.flags.n, true);
    assert_eq!(c.reg.flags.p, true);
    assert_eq!(c.reg.flags.c, false); // C unchanged
}

// CPIR terminated because BC reaches 0 with no match: Z=0, P=0
#[test]
fn cpir_bc_exhausted() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB1);
    c.reg.a = 0xAA;
    c.reg.set_hl(0x1000);
    c.reg.set_bc(0x03);
    b.write_byte(0x1000, 0x11);
    b.write_byte(0x1001, 0x22);
    b.write_byte(0x1002, 0x33); // none equal 0xAA
    run_block(&mut c, &mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x1003); // advanced by 3
    assert_eq!(c.reg.get_bc(), 0x00); // exhausted
    assert_eq!(c.reg.flags.z, false); // no match found
    assert_eq!(c.reg.flags.p, false); // BC=0
    assert_eq!(c.reg.flags.n, true);
}

// CPIR match found on very first iteration
#[test]
fn cpir_first_match() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB1);
    c.reg.a = 0x55;
    c.reg.set_hl(0x3000);
    c.reg.set_bc(0x05);
    b.write_byte(0x3000, 0x55); // immediate match
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x3001); // one increment
    assert_eq!(c.reg.get_bc(), 0x04); // one decrement (5-1=4)
    assert_eq!(c.reg.flags.z, true); // match
    assert_eq!(c.reg.flags.p, true); // BC=4 ≠ 0
    assert_eq!(c.reg.flags.n, true);
}

// CPDR terminated because BC reaches 0 with no match: Z=0, P=0
#[test]
fn cpdr_bc_exhausted() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB9);
    c.reg.a = 0xAA;
    c.reg.set_hl(0x1002);
    c.reg.set_bc(0x03);
    b.write_byte(0x1000, 0x11);
    b.write_byte(0x1001, 0x22);
    b.write_byte(0x1002, 0x33); // none equal 0xAA
    run_block(&mut c, &mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x0FFF); // decremented by 3
    assert_eq!(c.reg.get_bc(), 0x00); // exhausted
    assert_eq!(c.reg.flags.z, false); // no match found
    assert_eq!(c.reg.flags.p, false); // BC=0
    assert_eq!(c.reg.flags.n, true);
}

// CPDR match found on very first iteration
#[test]
fn cpdr_first_match() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0x0000, 0xED);
    b.write_byte(0x0001, 0xB9);
    c.reg.a = 0x77;
    c.reg.set_hl(0x3000);
    c.reg.set_bc(0x05);
    b.write_byte(0x3000, 0x77); // immediate match
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 2);
    assert_eq!(c.reg.get_hl(), 0x2FFF); // one decrement
    assert_eq!(c.reg.get_bc(), 0x04); // one decrement (5-1=4)
    assert_eq!(c.reg.flags.z, true); // match
    assert_eq!(c.reg.flags.p, true); // BC=4 ≠ 0
    assert_eq!(c.reg.flags.n, true);
}

// DJNZ inline tests (no bin fixture required)

#[test]
fn djnz_no_branch() {
    // B = 1: decrement gives 0, branch NOT taken → PC advances by 2, 8 cycles
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0100;
    c.reg.b = 0x01;
    b.write_byte(0x0100, 0x10); // DJNZ
    b.write_byte(0x0101, 0x05); // displacement +5 (ignored when no branch)
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.b, 0x00);
    assert_eq!(c.reg.pc, 0x0102);
}

#[test]
fn djnz_branch_positive() {
    // B = 3: decrement gives 2, branch taken with positive displacement +3
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0100;
    c.reg.b = 0x03;
    b.write_byte(0x0100, 0x10); // DJNZ
    b.write_byte(0x0101, 0x03); // displacement +3
    // Expected: PC = 0x0100 + 2 + 3 = 0x0105
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(c.reg.b, 0x02);
    assert_eq!(c.reg.pc, 0x0105);
}

#[test]
fn djnz_branch_negative() {
    // B = 2: decrement gives 1, branch taken with negative displacement -5 (0xFB)
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0110;
    c.reg.b = 0x02;
    b.write_byte(0x0110, 0x10); // DJNZ
    b.write_byte(0x0111, 0xFB); // displacement -5
    // signed_to_abs(0xFB) = !0xFB + 1 = 0x04 + 1 = 5
    // Expected: PC = 0x0110 + 2 - 5 = 0x010D
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(c.reg.b, 0x01);
    assert_eq!(c.reg.pc, 0x010D);
}

#[test]
fn djnz_b_wraps() {
    // B = 0: wrapping_sub(1) gives 0xFF, branch IS taken
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0100;
    c.reg.b = 0x00;
    b.write_byte(0x0100, 0x10); // DJNZ
    b.write_byte(0x0101, 0x02); // displacement +2
    // Expected: B = 0xFF, PC = 0x0100 + 2 + 2 = 0x0104, 13 cycles
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(c.reg.b, 0xFF);
    assert_eq!(c.reg.pc, 0x0104);
}

#[test]
fn djnz_max_positive_disp() {
    // Maximum positive displacement: +127 (0x7F)
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0100;
    c.reg.b = 0x02;
    b.write_byte(0x0100, 0x10); // DJNZ
    b.write_byte(0x0101, 0x7F); // displacement +127
    // Expected: PC = 0x0100 + 2 + 127 = 0x0181
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(c.reg.b, 0x01);
    assert_eq!(c.reg.pc, 0x0181);
}

#[test]
fn djnz_max_negative_disp() {
    // Maximum negative displacement: -128 (0x80)
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0200;
    c.reg.b = 0x02;
    b.write_byte(0x0200, 0x10); // DJNZ
    b.write_byte(0x0201, 0x80); // displacement -128
    // signed_to_abs(0x80) = !0x80 + 1 = 0x7F + 1 = 128
    // Expected: PC = 0x0200 + 2 - 128 = 0x0182
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(c.reg.b, 0x01);
    assert_eq!(c.reg.pc, 0x0182);
}

#[test]
fn djnz_loop() {
    // Full loop: B starts at 3, DJNZ with -2 displacement loops back to itself
    // until B reaches 0
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    c.reg.pc = 0x0100;
    c.reg.b = 0x03;
    b.write_byte(0x0100, 0x10); // DJNZ
    b.write_byte(0x0101, 0xFE); // displacement -2 (self-loop)

    // Iteration 1: B 3→2, branch taken, PC stays at 0x0100
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(c.reg.b, 0x02);
    assert_eq!(c.reg.pc, 0x0100);

    // Iteration 2: B 2→1, branch taken, PC stays at 0x0100
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(c.reg.b, 0x01);
    assert_eq!(c.reg.pc, 0x0100);

    // Iteration 3: B 1→0, branch NOT taken, PC advances to 0x0102
    assert_eq!(c.execute(&mut b), 8);
    assert_eq!(c.reg.b, 0x00);
    assert_eq!(c.reg.pc, 0x0102);
}

#[test]
fn djnz_dasm() {
    // Verify the disassembler output for DJNZ with positive and negative displacements
    let mut b = FlatBus::new(0xFFFF);

    // Positive displacement +3 at 0x0100: target = 0x0100 + 2 + 3 = 0x0105
    b.write_byte(0x0100, 0x10);
    b.write_byte(0x0101, 0x03);
    assert_eq!(
        crate::dasm::dasm(&mut b, 0x0100),
        (String::from("10 03         DJNZ $0105"), 2)
    );

    // Negative displacement -2 (0xFE) at 0x0200: target = 0x0200 + 2 - 2 = 0x0200
    b.write_byte(0x0200, 0x10);
    b.write_byte(0x0201, 0xFE);
    assert_eq!(
        crate::dasm::dasm(&mut b, 0x0200),
        (String::from("10 FE         DJNZ $0200"), 2)
    );
}

// --- Amstrad CPC interrupt-critical sequence -------------------------------
//
// The following routine comes from Amstrad CPC code, where both the timing and
// the exact semantics of the DI / EX AF,AF' / EXX / EI / EX AF,AF' / DI chain
// are critical (the CPC raises an interrupt every 300 microseconds / 52 scanlines):
//
//   B941  F3        DI
//   B942  08        EX AF,AF'
//   B943  38 33     JR C,$B978
//   B945  D9        EXX
//   B946  79        LD A,C
//   B947  37        SCF
//   B948  FB        EI
//   B949  08        EX AF,AF'
//   B94A  F3        DI
//   B94B  F5        PUSH AF
//
// Note that the JR C tests the carry flag of the *alternate* AF (it has just
// been swapped in), and that the EI ... DI window is exactly one instruction
// wide: since EI only takes effect after the instruction following it, the
// single spot where an interrupt can be acknowledged is right after the
// EX AF,AF' at B949, i.e. just before the DI at B94A.

const CPC_SEQ_ORG: u16 = 0xB941;
const CPC_SEQ: [u8; 11] = [
    0xF3, // DI
    0x08, // EX AF,AF'
    0x38, 0x33, // JR C,$B978
    0xD9, // EXX
    0x79, // LD A,C
    0x37, // SCF
    0xFB, // EI
    0x08, // EX AF,AF'
    0xF3, // DI
    0xF5, // PUSH AF
];

// Loads the sequence at B941, preceded by IM 1 / EI / NOP / NOP at B93C so that
// maskable interrupts are already enabled (and the EI delay expired) when the
// leading DI is reached.
fn cpc_seq_bus() -> FlatBus {
    let mut b = FlatBus::new(0xFFFF);
    b.write_byte(0xB93C, 0xED); // IM 1
    b.write_byte(0xB93D, 0x56);
    b.write_byte(0xB93E, 0xFB); // EI
    b.write_byte(0xB93F, 0x00); // NOP
    b.write_byte(0xB940, 0x00); // NOP
    for (i, byte) in CPC_SEQ.iter().enumerate() {
        b.write_byte(CPC_SEQ_ORG + i as u16, *byte);
    }
    b
}

// Runs IM 1 / EI / NOP / NOP, leaving PC on the leading DI with IFF1 set.
fn cpc_seq_prologue(c: &mut CPU, b: &mut FlatBus) {
    c.reg.pc = 0xB93C;
    c.reg.sp = 0x2000;
    for _ in 0..4 {
        c.execute(b);
    }
    assert_eq!(c.reg.pc, CPC_SEQ_ORG);
    assert!(c.iff1());
    assert_eq!(c.im(), 1);
}

// Nominal path (carry clear in AF'): the whole sequence is executed, exchanging
// both register banks twice and leaving interrupts disabled on exit.
#[test]
fn cpc_di_ex_exx_ei_sequence() {
    let mut c = CPU::new();
    let mut b = cpc_seq_bus();
    cpc_seq_prologue(&mut c, &mut b);

    c.reg.set_af(0x1100);
    c.alt.set_af(0x2200); // carry clear => JR C not taken
    c.reg.set_bc(0x1122);
    c.reg.set_de(0x3344);
    c.reg.set_hl(0x5566);
    c.alt.set_bc(0x9ABC);
    c.alt.set_de(0xDEF0);
    c.alt.set_hl(0x1357);

    // DI
    assert_eq!(c.execute(&mut b), 4);
    assert!(!c.iff1());
    assert!(!c.iff2());
    assert_eq!(c.reg.pc, 0xB942);

    // EX AF,AF'
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.get_af(), 0x2200);
    assert_eq!(c.alt.get_af(), 0x1100);
    assert_eq!(c.reg.pc, 0xB943);

    // JR C,$B978 : carry comes from the AF' just swapped in => not taken (7 cycles)
    assert_eq!(c.execute(&mut b), 7);
    assert_eq!(c.reg.pc, 0xB945);

    // EXX
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.get_bc(), 0x9ABC);
    assert_eq!(c.reg.get_de(), 0xDEF0);
    assert_eq!(c.reg.get_hl(), 0x1357);
    assert_eq!(c.alt.get_bc(), 0x1122);
    assert_eq!(c.alt.get_de(), 0x3344);
    assert_eq!(c.alt.get_hl(), 0x5566);
    // EXX must not touch AF / AF'
    assert_eq!(c.reg.get_af(), 0x2200);
    assert_eq!(c.alt.get_af(), 0x1100);
    assert_eq!(c.reg.pc, 0xB946);

    // LD A,C : C is the alternate C brought in by EXX
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.a, 0xBC);
    assert_eq!(c.reg.pc, 0xB947);

    // SCF
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.flags(), CF);
    assert_eq!(c.reg.pc, 0xB948);

    // EI : IFF1/IFF2 set, but interrupts are not acknowledged yet
    assert_eq!(c.execute(&mut b), 4);
    assert!(c.iff1());
    assert!(c.iff2());
    assert_eq!(c.reg.pc, 0xB949);

    // EX AF,AF' : back to the caller's AF, the modified one is parked in AF'
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.reg.get_af(), 0x1100);
    assert_eq!(c.alt.get_af(), 0xBC00 | CF as u16);
    assert_eq!(c.reg.pc, 0xB94A);

    // DI : closes the one-instruction window opened by EI
    assert_eq!(c.execute(&mut b), 4);
    assert!(!c.iff1());
    assert!(!c.iff2());
    assert_eq!(c.reg.pc, 0xB94B);

    // PUSH AF : the caller's AF is the one saved
    assert_eq!(c.execute(&mut b), 11);
    assert_eq!(c.reg.sp, 0x1FFE);
    assert_eq!(b.read_word(0x1FFE), 0x1100);
    assert_eq!(c.reg.pc, 0xB94C);
}

// Branch path (carry set in AF'): the JR skips the EI, so the routine leaves
// with interrupts still disabled by the leading DI.
#[test]
fn cpc_di_ex_exx_ei_sequence_jr_taken() {
    let mut c = CPU::new();
    let mut b = cpc_seq_bus();
    b.write_byte(0xB978, 0x00); // NOP at the JR target
    cpc_seq_prologue(&mut c, &mut b);

    c.reg.set_af(0x1100);
    c.alt.set_af(0x2200 | CF as u16); // carry set => JR C taken

    // DI
    assert_eq!(c.execute(&mut b), 4);
    assert!(!c.iff1());

    // EX AF,AF' brings in the alternate AF, whose carry is set
    assert_eq!(c.execute(&mut b), 4);
    assert_eq!(c.flags() & CF, CF);
    assert_eq!(c.reg.pc, 0xB943);

    // JR C,$B978 taken : 12 cycles
    assert_eq!(c.execute(&mut b), 12);
    assert_eq!(c.reg.pc, 0xB978);

    // The EI at B948 has been skipped: interrupts stay disabled
    assert!(!c.iff1());
    assert!(!c.iff2());
    c.int_request(0x00);
    assert_eq!(c.execute(&mut b), 4); // NOP, no interrupt acknowledged
    assert_eq!(c.reg.pc, 0xB979);
    assert!(c.has_pending_int()); // still latched
}

// An interrupt raised while the sequence runs with interrupts disabled must stay
// latched, and be acknowledged at exactly one point: after the EX AF,AF' that
// follows the EI, i.e. instead of the DI at B94A.
#[test]
fn cpc_int_acknowledged_in_the_ei_di_window() {
    let mut c = CPU::new();
    let mut b = cpc_seq_bus();
    cpc_seq_prologue(&mut c, &mut b);

    c.reg.set_af(0x1100);
    c.alt.set_af(0x2200); // carry clear => straight line path
    c.alt.set_bc(0x9ABC);

    // DI
    c.execute(&mut b);
    assert!(!c.iff1());

    // The CPC raster interrupt fires while interrupts are masked.
    c.int_request(0x00); // in IM 1 the data bus value is ignored (RST 38 is forced)

    // EX AF,AF' / JR C / EXX / LD A,C / SCF / EI must all run undisturbed,
    // the request staying latched until it can be served.
    for expected_pc in [0xB943, 0xB945, 0xB946, 0xB947, 0xB948, 0xB949] {
        c.execute(&mut b);
        assert_eq!(c.reg.pc, expected_pc);
        assert!(c.has_pending_int());
    }
    assert!(c.iff1()); // EI has been executed...

    // ...but its effect is delayed by one instruction: the EX AF,AF' at B949 runs.
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0xB94A);
    assert_eq!(c.reg.get_af(), 0x1100);
    assert!(c.has_pending_int());

    // Now the interrupt is acknowledged, in place of the DI at B94A:
    // IM 1 forces a RST 38, 11 T-states plus the two wait states of the acknowledge cycle.
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(c.reg.pc, 0x0038);
    assert_eq!(c.reg.sp, 0x1FFE);
    assert_eq!(b.read_word(0x1FFE), 0xB94A); // the DI has not been executed yet
    assert!(!c.has_pending_int());
    // Acknowledging a maskable interrupt clears both flip-flops
    assert!(!c.iff1());
    assert!(!c.iff2());
}

// An interrupt raised after the closing DI must never be served: it stays
// latched until the program re-enables interrupts.
#[test]
fn cpc_int_raised_after_closing_di_stays_latched() {
    let mut c = CPU::new();
    let mut b = cpc_seq_bus();
    // Tail after the sequence: NOP / NOP / EI / NOP
    b.write_byte(0xB94C, 0x00);
    b.write_byte(0xB94D, 0x00);
    b.write_byte(0xB94E, 0xFB);
    b.write_byte(0xB94F, 0x00);
    cpc_seq_prologue(&mut c, &mut b);

    c.reg.set_af(0x1100);
    c.alt.set_af(0x2200);
    c.alt.set_bc(0x9ABC);

    // Run the whole sequence up to and including the closing DI at B94A.
    for _ in 0..9 {
        c.execute(&mut b);
    }
    assert_eq!(c.reg.pc, 0xB94B);
    assert!(!c.iff1());
    assert!(!c.iff2());

    c.int_request(0x00);

    // PUSH AF, NOP, NOP: nothing is acknowledged while IFF1 is clear.
    for expected_pc in [0xB94C, 0xB94D, 0xB94E] {
        c.execute(&mut b);
        assert_eq!(c.reg.pc, expected_pc);
        assert!(c.has_pending_int());
    }
    assert_eq!(b.read_word(0x1FFE), 0x1100); // PUSH AF did happen

    // EI at B94E: still delayed by one instruction.
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0xB94F);
    assert!(c.iff1());
    assert!(c.has_pending_int());

    // NOP at B94F runs, then the latched request is finally served.
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0xB950);
    assert!(c.has_pending_int());
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x0038);
    assert_eq!(b.read_word(0x1FFC), 0xB950);
    assert!(!c.has_pending_int());
}

// --- Assembled interrupt programs (tests/int*.asm) -------------------------
//
// These three programs exercise a full round trip through an assembled binary:
// the main loop spins on CP B / JP NZ until the interrupt service routine loads
// A (0x0F) into B, then falls through to a RET that pops 0x0000 off the stack.
// Reaching PC 0x0000 with the stack unwound is the success condition.

// tests/int.asm, IM 0: the RST 08 opcode placed on the data bus is executed as-is.
#[test]
fn int_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/int.bin", 0).unwrap();

    // LD SP,0xFF00 / LD A,0x0F / JP start / EI / CP B / JP NZ,@loop
    for _ in 0..6 {
        c.execute(&mut b);
    }
    assert_eq!(c.reg.sp, 0xFF00);
    assert_eq!(c.reg.a, 0x0F);
    assert_eq!(c.reg.pc, 0x0011); // spinning on @loop
    assert!(c.iff1());

    c.int_request(0xCF); // RST 08

    // Acknowledge: 11 (RST 08) + 2 wait states, return address pushed, handler at 0x0008
    assert_eq!(c.execute(&mut b), 13);
    assert_eq!(c.reg.pc, 0x0008);
    assert_eq!(c.reg.sp, 0xFEFE);
    assert_eq!(b.read_word(0xFEFE), 0x0011);
    assert!(!c.iff1()); // acknowledging clears both flip-flops
    assert!(!c.iff2());

    c.execute(&mut b); // LD B,A
    assert_eq!(c.reg.b, 0x0F);
    c.execute(&mut b); // RET
    assert_eq!(c.reg.pc, 0x0011);
    assert_eq!(c.reg.sp, 0xFF00);

    // CP B now sets Z, the loop is left and the final RET returns to 0x0000
    c.execute(&mut b); // CP B
    assert_eq!(c.flags() & ZF, ZF);
    c.execute(&mut b); // JP NZ,@loop (not taken)
    assert_eq!(c.reg.pc, 0x0015);
    c.execute(&mut b); // RET
    assert_eq!(c.reg.pc, 0x0000);
    assert_eq!(c.reg.sp, 0xFF02);
}

// tests/int_im1.asm, IM 1: the vector put on the data bus is ignored, a RST 38 is
// always forced. The program has a RST 18 handler at 0x0018 (which loads C instead
// of B) precisely to catch an implementation that would honour the bus value.
#[test]
fn int_im1_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/int_im1.bin", 0).unwrap();

    // LD SP,0xFF00 / LD A,0x0F / JP start / IM 1 / EI / CP B / JP NZ,@loop
    for _ in 0..7 {
        c.execute(&mut b);
    }
    assert_eq!(c.im(), 1);
    assert_eq!(c.reg.pc, 0x0053); // spinning on @loop
    assert!(c.iff1());

    c.int_request(0xDF); // RST 18: must be ignored in IM 1

    assert_eq!(c.execute(&mut b), 13); // forced RST 38: 11 + 2 wait states
    assert_eq!(c.reg.pc, 0x0038);
    assert_eq!(b.read_word(0xFEFE), 0x0053);

    c.execute(&mut b); // LD B,A
    c.execute(&mut b); // RET
    assert_eq!(c.reg.b, 0x0F);
    assert_eq!(c.reg.c, 0x00); // the 0x0018 handler has not been entered
    assert_eq!(c.reg.pc, 0x0053);

    c.execute(&mut b); // CP B
    c.execute(&mut b); // JP NZ,@loop (not taken)
    assert_eq!(c.reg.pc, 0x0057);
    c.execute(&mut b); // RET
    assert_eq!(c.reg.pc, 0x0000);
    assert_eq!(c.reg.sp, 0xFF02);
}

// tests/int_im2.asm, IM 2: I (0x01) and the bus vector (0x02) build the address of
// the jump table entry at 0x0102, which points at the handler at 0x0106. The dummy
// handler at 0x0038 (loading D) catches an implementation falling back to IM 1.
#[test]
fn int_im2_asm() {
    let mut c = CPU::new();
    let mut b = FlatBus::new(0xFFFF);
    b.load_bin("bin/int_im2.bin", 0).unwrap();

    // LD SP,0xFF00 / LD A,0x01 / LD I,A / LD A,0x0F / JP start / IM 2 / EI / CP B / JP NZ,@loop
    for _ in 0..9 {
        c.execute(&mut b);
    }
    assert_eq!(c.im(), 2);
    assert_eq!(c.reg.i, 0x01);
    assert_eq!(c.reg.pc, 0x0053); // spinning on @loop
    assert!(c.iff1());

    c.int_request(0x02); // vector: table entry at (I << 8) | 0x02 = 0x0102

    assert_eq!(c.execute(&mut b), 19); // IM 2 acknowledge
    assert_eq!(c.reg.pc, 0x0106); // read from the jump table
    assert_eq!(b.read_word(0xFEFE), 0x0053);

    c.execute(&mut b); // LD B,A
    c.execute(&mut b); // RET
    assert_eq!(c.reg.b, 0x0F);
    assert_eq!(c.reg.d, 0x00); // the 0x0038 handler has not been entered
    assert_eq!(c.reg.pc, 0x0053);

    c.execute(&mut b); // CP B
    c.execute(&mut b); // JP NZ,@loop (not taken)
    assert_eq!(c.reg.pc, 0x0057);
    c.execute(&mut b); // RET
    assert_eq!(c.reg.pc, 0x0000);
    assert_eq!(c.reg.sp, 0xFF02);
}

/// Les instructions de sortie par bloc décrémentent B AVANT de présenter le
/// port sur le bus d'adresse : c'est donc B-1 qui apparaît sur A8-A15. Sur une
/// machine dont l'octet de poids fort sélectionne le périphérique (Amstrad CPC),
/// se tromper d'un cran envoie l'écriture au mauvais registre matériel.
#[test]
fn out_block_instructions_decrement_b_before_the_port_access() {
    struct IoLog {
        ram: [u8; 0x10000],
        writes: Vec<(u16, u8)>,
    }
    impl Bus for IoLog {
        fn read_byte(&self, a: u16) -> u8 {
            self.ram[a as usize]
        }
        fn write_byte(&mut self, a: u16, v: u8) {
            self.ram[a as usize] = v;
        }
        fn write_io(&mut self, port: u16, data: u8) {
            self.writes.push((port, data));
        }
    }

    // L'idiome de l'époque : INC B compense la décrémentation de OUTI, si bien
    // que le port visé est bien 0xBDxx et non 0xBCxx.
    //   LD B,0xBC / LD C,0x00 / LD HL,0x0100 / INC B / OUTI / INC B / OUTI
    let mut b = IoLog {
        ram: [0; 0x10000],
        writes: Vec::new(),
    };
    for (i, byte) in [
        0x06, 0xBC, 0x0E, 0x00, 0x21, 0x00, 0x01, 0x04, 0xED, 0xA3, 0x04, 0xED, 0xA3,
    ]
    .iter()
    .enumerate()
    {
        b.ram[i] = *byte;
    }
    b.ram[0x0100] = 0xAA;
    b.ram[0x0101] = 0x55;

    let mut c = CPU::new();
    for _ in 0..7 {
        c.execute(&mut b);
    }

    assert_eq!(b.writes, vec![(0xBC00, 0xAA), (0xBC00, 0x55)]);
    assert_eq!(c.reg.b, 0xBC);
    assert_eq!(c.reg.get_hl(), 0x0102);
}

/// À l'inverse, les instructions d'entrée par bloc décrémentent B APRÈS l'accès
/// et présentent donc B inchangé sur le bus d'adresse.
#[test]
fn in_block_instructions_decrement_b_after_the_port_access() {
    struct IoLog {
        ram: [u8; 0x10000],
        reads: Vec<u16>,
    }
    impl Bus for IoLog {
        fn read_byte(&self, a: u16) -> u8 {
            self.ram[a as usize]
        }
        fn write_byte(&mut self, a: u16, v: u8) {
            self.ram[a as usize] = v;
        }
        fn read_io(&self, port: u16) -> u8 {
            // read_io prend &self : on ne peut pas journaliser ici, on vérifie
            // donc la valeur lue en la faisant dépendre du port.
            (port >> 8) as u8
        }
    }

    // LD B,0x10 / LD C,0x00 / LD HL,0x0100 / INI
    let mut b = IoLog {
        ram: [0; 0x10000],
        reads: Vec::new(),
    };
    for (i, byte) in [0x06, 0x10, 0x0E, 0x00, 0x21, 0x00, 0x01, 0xED, 0xA2]
        .iter()
        .enumerate()
    {
        b.ram[i] = *byte;
    }

    let mut c = CPU::new();
    for _ in 0..4 {
        c.execute(&mut b);
    }

    // L'octet rangé en 0x0100 est l'image du poids fort du port : B non décrémenté.
    assert_eq!(b.ram[0x0100], 0x10);
    assert_eq!(c.reg.b, 0x0F);
    let _ = &b.reads;
}

/// Le PC d'un Z80 reboucle de 0xFFFF à 0x0000 : aucune adresse ne peut faire
/// fauter le processeur. Un programme émulé parti à la dérive doit donc mener
/// l'émulateur n'importe où plutôt que de le faire paniquer, ce qui laisse une
/// chance de l'observer au débogueur.
#[test]
fn program_counter_wraps_around_instead_of_overflowing() {
    let mut b = FlatBus::new(0xFFFF);
    let mut c = CPU::new();

    // NOP en toute fin d'espace d'adressage.
    b.write_byte(0xFFFF, 0x00);
    c.reg.pc = 0xFFFF;
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x0000);

    // RST 38 (0xFF, l'octet de remplissage typique) hors contexte d'interruption :
    // c'est ce que rencontre un programme qui saute dans de la mémoire vierge.
    b.write_byte(0xFFFF, 0xFF);
    c.reg.pc = 0xFFFF;
    c.reg.sp = 0x0001;
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x0038);

    // JP nn dont l'opérande est à cheval sur la fin de l'espace d'adressage.
    b.write_byte(0xFFFF, 0xC3);
    b.write_byte(0x0000, 0x34);
    b.write_byte(0x0001, 0x12);
    c.reg.pc = 0xFFFF;
    c.execute(&mut b);
    assert_eq!(c.reg.pc, 0x1234);
}

/// Un HALT interrompu doit reprendre APRÈS le HALT, pas dessus. Le PC reste sur
/// l'opcode HALT pendant l'attente ; s'il n'avance pas au moment où
/// l'interruption est acceptée, l'adresse de retour empilée est celle du HALT
/// lui-même et le programme y retombe à chaque interruption, définitivement.
/// C'est le mode de synchronisation trame de la plupart des jeux.
#[test]
fn halt_resumes_after_the_halt_instruction() {
    let mut b = FlatBus::new(0xFFFF);
    let mut c = CPU::new();

    b.write_byte(0x00FD, 0xED); // IM 1
    b.write_byte(0x00FE, 0x56);
    b.write_byte(0x00FF, 0xFB); // EI
    b.write_byte(0x0100, 0x76); // HALT
    b.write_byte(0x0101, 0x3C); // INC A
    b.write_byte(0x0038, 0xFB); // handler : EI
    b.write_byte(0x0039, 0xC9); //           RET

    c.reg.pc = 0x00FD;
    c.reg.sp = 0xFF00;
    c.execute(&mut b); // IM 1
    c.execute(&mut b); // EI
    assert_eq!(c.reg.pc, 0x0100);

    c.execute(&mut b); // HALT
    assert!(c.is_halted());
    assert_eq!(c.reg.pc, 0x0100);

    // Sans interruption, le CPU patiente sur place.
    c.execute(&mut b);
    assert!(c.is_halted());
    assert_eq!(c.reg.pc, 0x0100);

    c.int_request(0xFF);
    c.execute(&mut b); // acquittement : RST 38
    assert!(!c.is_halted());
    assert_eq!(c.reg.pc, 0x0038);
    // L'adresse empilée doit être celle de l'instruction qui SUIT le HALT.
    assert_eq!(b.read_word(c.reg.sp), 0x0101);

    c.execute(&mut b); // EI
    c.execute(&mut b); // RET
    assert_eq!(c.reg.pc, 0x0101);

    c.execute(&mut b); // INC A : le programme a bien repris son cours
    assert_eq!(c.reg.a, 0x01);
}

/// Les formes non documentées de DD CB / FD CB : l'opération porte sur la
/// case mémoire indexée, et son résultat est aussi rangé dans le registre que
/// désigne le champ z. Elles étaient absentes, et une instruction absente
/// coûtait 255 cycles de temps émulé.
#[test]
fn indexed_bit_operations_also_copy_into_a_register() {
    // RLC (IX+2) avec copie dans B, C ... A
    for (z, name) in [
        (0u8, "B"),
        (1, "C"),
        (2, "D"),
        (3, "E"),
        (4, "H"),
        (5, "L"),
        (7, "A"),
    ] {
        let mut c = CPU::new();
        let mut b = FlatBus::new(0xFFFF);
        b.write_byte(0x0000, 0xDD);
        b.write_byte(0x0001, 0xCB);
        b.write_byte(0x0002, 0x02);
        b.write_byte(0x0003, 0x00 | z); // RLC (IX+2) -> registre z
        c.reg.set_ix(0x9000);
        b.write_byte(0x9002, 0b1000_0001);

        let cycles = c.execute(&mut b);

        let expected = 0b0000_0011; // rotation a gauche
        assert_eq!(b.read_byte(0x9002), expected, "memoire, copie vers {name}");
        let got = match z {
            0 => c.reg.b,
            1 => c.reg.c,
            2 => c.reg.d,
            3 => c.reg.e,
            4 => c.reg.h,
            5 => c.reg.l,
            _ => c.reg.a,
        };
        assert_eq!(got, expected, "registre {name}");
        assert_eq!(cycles, 23, "duree de la forme vers {name}");
        assert_eq!(c.reg.pc, 4);
    }
}

#[test]
fn indexed_res_and_set_reach_every_bit_and_both_index_registers() {
    for bit in 0..8u8 {
        let mut c = CPU::new();
        let mut b = FlatBus::new(0xFFFF);
        b.write_byte(0x0000, 0xFD);
        b.write_byte(0x0001, 0xCB);
        b.write_byte(0x0002, 0xFE); // deplacement -2
        b.write_byte(0x0003, 0xC6 | (bit << 3)); // SET bit,(IY-2)
        c.reg.set_iy(0x9000);
        b.write_byte(0x8FFE, 0x00);

        assert_eq!(c.execute(&mut b), 23);
        assert_eq!(b.read_byte(0x8FFE), 1 << bit, "SET {bit},(IY-2)");

        // Et l'inverse : RES le remet a zero.
        b.write_byte(0x0003, 0x86 | (bit << 3));
        c.reg.pc = 0;
        assert_eq!(c.execute(&mut b), 23);
        assert_eq!(b.read_byte(0x8FFE), 0x00, "RES {bit},(IY-2)");
    }
}

/// BIT n'écrit nulle part, quel que soit le champ z : c'est la seule des
/// quatre familles à ne rien recopier.
#[test]
fn indexed_bit_test_writes_nothing() {
    for z in 0..8u8 {
        let mut c = CPU::new();
        let mut b = FlatBus::new(0xFFFF);
        b.write_byte(0x0000, 0xDD);
        b.write_byte(0x0001, 0xCB);
        b.write_byte(0x0002, 0x02);
        b.write_byte(0x0003, 0x40 | z); // BIT 0,(IX+2)
        c.reg.set_ix(0x9000);
        b.write_byte(0x9002, 0x01);
        c.reg.b = 0xAA;

        assert_eq!(c.execute(&mut b), 20, "BIT dure 20 cycles");
        assert_eq!(b.read_byte(0x9002), 0x01, "la memoire ne doit pas bouger");
        assert_eq!(c.reg.b, 0xAA, "aucun registre ne doit bouger");
        assert!(!c.reg.flags.z, "le bit 0 est a 1, donc Z est faux");
    }
}
