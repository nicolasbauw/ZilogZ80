use crate::cpu::CPU;

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
        c.ix = 0x2233;
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
    fn chk_add_overflow() {
        assert_eq!(crate::cpu::check_add_overflow(120, 105), true);
    }

    #[test]
    fn chk_sub_overflow() {
        assert_eq!(crate::cpu::check_sub_overflow(129, 10), true);
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