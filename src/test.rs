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
    c.execute();
    assert_eq!(c.registers.b, 0x11);
    c.execute();
    assert_eq!(c.registers.b, 0x15);
    c.execute();
    assert_eq!(c.registers.b, 0x1f);
    c.execute();
    assert_eq!(c.registers.b, 0x21);
    c.execute();
    assert_eq!(c.registers.b, 0x25);
    c.execute();
    assert_eq!(c.registers.b, 0x2f);
    c.execute();
    assert_eq!(c.registers.b, 0x31);
    c.execute();
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
    c.execute();
    assert_eq!(c.registers.c, 0x11);
    c.execute();
    assert_eq!(c.registers.c, 0x11);
    c.execute();
    assert_eq!(c.registers.c, 0x1f);
    c.execute();
    assert_eq!(c.registers.c, 0x21);
    c.execute();
    assert_eq!(c.registers.c, 0x25);
    c.execute();
    assert_eq!(c.registers.c, 0x2f);
    c.execute();
    assert_eq!(c.registers.c, 0x31);
    c.execute();
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
    c.execute();
    assert_eq!(c.registers.d, 0x11);
    c.execute();
    assert_eq!(c.registers.d, 0x15);
    c.execute();
    assert_eq!(c.registers.d, 0x15);
    c.execute();
    assert_eq!(c.registers.d, 0x21);
    c.execute();
    assert_eq!(c.registers.d, 0x25);
    c.execute();
    assert_eq!(c.registers.d, 0x2f);
    c.execute();
    assert_eq!(c.registers.d, 0x31);
    c.execute();
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
        c.execute();
        assert_eq!(c.registers.e, 0x11);
        c.execute();
        assert_eq!(c.registers.e, 0x15);
        c.execute();
        assert_eq!(c.registers.e, 0x1f);
        c.execute();
        assert_eq!(c.registers.e, 0x1f);
        c.execute();
        assert_eq!(c.registers.e, 0x25);
        c.execute();
        assert_eq!(c.registers.e, 0x2f);
        c.execute();
        assert_eq!(c.registers.e, 0x31);
        c.execute();
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
        c.execute();
        assert_eq!(c.registers.h, 0x11);
        c.execute();
        assert_eq!(c.registers.h, 0x15);
        c.execute();
        assert_eq!(c.registers.h, 0x1f);
        c.execute();
        assert_eq!(c.registers.h, 0x21);
        c.execute();
        assert_eq!(c.registers.h, 0x21);
        c.execute();
        assert_eq!(c.registers.h, 0x2f);
        c.execute();
        assert_eq!(c.registers.h, 0x31);
        c.execute();
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
        c.execute();
        assert_eq!(c.registers.l, 0x11);
        c.execute();
        assert_eq!(c.registers.l, 0x15);
        c.execute();
        assert_eq!(c.registers.l, 0x1f);
        c.execute();
        assert_eq!(c.registers.l, 0x21);
        c.execute();
        assert_eq!(c.registers.l, 0x25);
        c.execute();
        assert_eq!(c.registers.l, 0x25);
        c.execute();
        assert_eq!(c.registers.l, 0x31);
        c.execute();
        assert_eq!(c.registers.l, 0x3f);
        assert_eq!(c.pc, 8);
    }

    #[test]
    fn ld_m() {
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
        c.execute();
        assert_eq!(c.bus.read_byte(0x252f), 0x11);
        c.execute();
        assert_eq!(c.bus.read_byte(0x252f), 0x15);
        c.execute();
        assert_eq!(c.bus.read_byte(0x252f), 0x1f);
        c.execute();
        assert_eq!(c.bus.read_byte(0x252f), 0x21);
        c.execute();
        assert_eq!(c.bus.read_byte(0x252f), 0x25);
        c.execute();
        assert_eq!(c.bus.read_byte(0x252f), 0x2f);
        c.execute();
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
        c.execute();
        assert_eq!(c.registers.a, 0x11);
        c.execute();
        assert_eq!(c.registers.a, 0x15);
        c.execute();
        assert_eq!(c.registers.a, 0x1f);
        c.execute();
        assert_eq!(c.registers.a, 0x21);
        c.execute();
        assert_eq!(c.registers.a, 0x25);
        c.execute();
        assert_eq!(c.registers.a, 0x2f);
        c.execute();
        assert_eq!(c.registers.a, 0x31);
        c.execute();
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