use crate::registers::Registers;
use crate::memory::AddressBus;
use crate::bit::get_bit;

const CYCLES: [u8; 256] = [
    0, 10, 0, 6, 4, 4, 7, 4, 4, 11, 7, 6, 4, 4, 7, 4,
    0, 10, 0, 6, 4, 4, 7, 4, 0, 11, 7, 6, 4, 4, 7, 4,
    0, 10, 16, 6, 4, 4, 7, 0, 0, 11, 16, 6, 4, 4, 7, 4,
    0, 10, 13, 6, 11, 11, 7, 4, 0, 11, 0, 6, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    7, 7, 7, 7, 7, 7, 4, 7, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    0, 10, 0, 0, 0, 11, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0,
    0, 10, 0, 0, 0, 11, 7, 0, 0, 4, 0, 0, 0, 0, 7, 0,
    0, 10, 0, 19, 0, 11, 7, 0, 0, 0, 0, 4, 0, 0, 7, 0,
    0, 10, 0, 4, 0, 11, 7, 0, 0, 6, 0, 4, 0, 0, 7, 0,
];

const CYCLES_DD: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0,
    0, 14, 20, 10, 0, 0, 0, 0, 0, 15, 20, 10, 0, 0, 0, 0,
    0, 0, 0, 0, 23, 23, 19, 0, 0, 15, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    19, 19, 19, 19, 19, 19, 0, 19, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 14, 0, 23, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0,
];

const CYCLES_FD: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0,
    0, 14, 20, 10, 0, 0, 0, 0, 0, 15, 20, 10, 0, 0, 0, 0,
    0, 0, 0, 0, 23, 23, 19, 0, 0, 15, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    19, 19, 19, 19, 19, 19, 0, 19, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 14, 0, 23, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0,
];

const CYCLES_ED: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 15, 20, 8, 0, 8, 9, 0, 0, 15, 20, 0, 0, 0, 9,
    0, 0, 15, 20, 0, 0, 8, 9, 0, 0, 15, 20, 0, 0, 8, 9,
    0, 0, 15, 20, 0, 0, 0, 0, 0, 0, 15, 20, 0, 0, 0, 0,
    0, 0, 15, 20, 0, 0, 0, 0, 0, 0, 15, 20, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    16, 16, 0, 0, 0, 0, 0, 0, 16, 16, 0, 0, 0, 0, 0, 0,
    21, 21, 0, 0, 0, 0, 0, 0, 21, 21, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// Check addition overflow
fn check_add_overflow(n1: u8, n2: u8) -> bool {
    let r = (n1 as i8).overflowing_add(n2 as i8);
    r.1
}

// Check substraction overflow
fn check_sub_overflow(n1: u8, n2: u8) -> bool {
    let r = (n1 as i8).overflowing_sub(n2 as i8);
    r.1
}

pub struct CPU {
    pub registers: Registers,
    pub alt_registers: Registers,
    pub i: u8,
    pub r: u8,
    pub ix: u16,
    pub iy: u16,
    pub sp: u16,
    pub pc: u16,
    pub bus: AddressBus,
    im: u8,
    pub halt: bool,
    iff1: bool,
    iff2: bool,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            registers: Registers::new(),
            alt_registers: Registers::new(),
            i: 0,
            r: 0,
            ix: 0,
            iy: 0,
            sp: 0,
            pc: 0,
            bus: AddressBus::new(),
            im: 0,
            halt: false,
            iff1: false,
            iff2: false,
        }
    }

    fn ldi(&mut self) {
        let bc = self.registers.get_bc();
        let de = self.registers.get_de();
        let hl = self.registers.get_hl();
        self.bus.write_byte(de, self.bus.read_byte(hl));
        self.registers.set_de(de.wrapping_add(1));
        self.registers.set_hl(hl.wrapping_add(1));
        self.registers.set_bc(bc.wrapping_sub(1));
    }

    fn ldd(&mut self) {
        let bc = self.registers.get_bc();
        let de = self.registers.get_de();
        let hl = self.registers.get_hl();
        self.bus.write_byte(de, self.bus.read_byte(hl));
        self.registers.set_de(de.wrapping_sub(1));
        self.registers.set_hl(hl.wrapping_sub(1));
        self.registers.set_bc(bc.wrapping_sub(1));
    }

    // Returns A - (HL)
    fn cpi(&mut self) -> u8 {
        let bc = self.registers.get_bc();
        let hl = self.registers.get_hl();
        let r = self.registers.a.wrapping_sub(self.bus.read_byte(hl));
        self.registers.set_hl(hl.wrapping_add(1));
        self.registers.set_bc(bc.wrapping_sub(1));
        r
    }

    // Returns A - (HL)
    fn cpd(&mut self) -> u8 {
        let bc = self.registers.get_bc();
        let hl = self.registers.get_hl();
        let r = self.registers.a.wrapping_sub(self.bus.read_byte(hl));
        self.registers.set_hl(hl.wrapping_sub(1));
        self.registers.set_bc(bc.wrapping_sub(1));
        r
    }

    // ADD A,r
    fn add(&mut self, n: u8)  {
        let a = self.registers.a;
        let r = a.wrapping_add(n);
        self.registers.flags.z = r == 0x00;
        self.registers.flags.s = (r as i8) < 0;
        self.registers.flags.p = check_add_overflow(self.registers.a, n);
        self.registers.flags.h = (a & 0x0f) + (n & 0x0f) > 0x0f;
        self.registers.flags.c = u16::from(a) + u16::from(n) > 0xff;
        self.registers.flags.n = false;
        self.registers.a = r;
    }

    // ADD A,s : ADD with carry
    fn addc(&mut self, n: u8)  {
        let c: u8 = match self.registers.flags.c {
            false => 0,
            true => 1,
        };
        let a = self.registers.a;
        let r = a.wrapping_add(n).wrapping_add(c);
        self.registers.flags.z = r == 0x00;
        self.registers.flags.s = (r as i8) < 0;
        self.registers.flags.p = check_add_overflow(self.registers.a, n.wrapping_add(c));
        self.registers.flags.h = (a & 0x0f) + (n & 0x0f) > 0x0f;
        self.registers.flags.c = u16::from(a) + u16::from(n) + u16::from(c) > 0xff;
        self.registers.flags.n = false;
        self.registers.a = r;
    }

    // SUB s
    fn sub(&mut self, n: u8)  {
        let a = self.registers.a;
        let r = a.wrapping_sub(n);
        self.registers.flags.z = r == 0x00;
        self.registers.flags.s = (r as i8) < 0;
        self.registers.flags.p = check_sub_overflow(self.registers.a, n);
        self.registers.flags.h = (a as i8 & 0x0f) - (n as i8 & 0x0f) >= 0x00;
        self.registers.flags.c = u16::from(a) < u16::from(n);
        self.registers.flags.n = true;
        self.registers.a = r;
    }

    // SBC s
    fn subc(&mut self, n: u8)  {
        let c: u8 = match self.registers.flags.c {
            false => 0,
            true => 1,
        };
        let a = self.registers.a;
        let r = a.wrapping_sub(n.wrapping_add(c));
        self.registers.flags.z = r == 0x00;
        self.registers.flags.s = (r as i8) < 0;
        self.registers.flags.p = check_sub_overflow(self.registers.a, n.wrapping_add(c));
        self.registers.flags.h = (a as i8 & 0x0f) - (n as i8 & 0x0f) >= 0x00;
        self.registers.flags.c = u16::from(a) < u16::from(n) + u16::from(c);
        self.registers.flags.n = true;
        self.registers.a = r;
    }

    // Logical AND
    fn and(&mut self, n: u8) {
        let r = self.registers.a & n;
        self.registers.flags.z = r == 0x00;
        self.registers.flags.s = (r as i8) < 0;
        self.registers.flags.p = r.count_ones() & 0x01 == 0x00;
        self.registers.flags.h = true;
        self.registers.flags.c = false;
        self.registers.flags.n = false;
        self.registers.a = r;
    }

    // Logical OR
    fn or(&mut self, n: u8) {
        let r = self.registers.a | n;
        self.registers.flags.z = r == 0x00;
        self.registers.flags.s = (r as i8) < 0;
        self.registers.flags.p = r.count_ones() & 0x01 == 0x00;
        self.registers.flags.h = false;
        self.registers.flags.c = false;
        self.registers.flags.n = false;
        self.registers.a = r;
    }
    
    // Logical exclusive-OR
    fn xor(&mut self, n: u8) {
        let a = self.registers.a;
        let r = a ^ n;
        self.registers.flags.z = r == 0x00;
        self.registers.flags.s = (r as i8) < 0;
        self.registers.flags.p = r.count_ones() & 0x01 == 0x00;
        self.registers.flags.h = false;
        self.registers.flags.c = false;
        self.registers.flags.n = false;
        self.registers.a = r;
    }

    // Comparison with accumulator
    fn cp(&mut self, n: u8) {
        let r = self.registers.a;
        self.sub(n);
        self.registers.a = r;
    }

    // Increment
    fn inc(&mut self, n: u8) -> u8 {
        let r = n.wrapping_add(1);
        self.registers.flags.z = r == 0x00;
        self.registers.flags.s = (r as i8) < 0;
        self.registers.flags.p = n == 0x7F;
        self.registers.flags.h = (n & 0x0f) + 0x01 > 0x0f;
        self.registers.flags.n = false;
        r
    }

    // Decrement
    fn dec(&mut self, n: u8) -> u8 {
        let r = n.wrapping_sub(1);
        self.registers.flags.z = r == 0x00;
        self.registers.flags.s = (r as i8) < 0;
        self.registers.flags.p = n == 0x80;
        self.registers.flags.h = (r & 0x0f) != 0x0f;
        self.registers.flags.n = true;
        r
    }

    // Decimal adjust accumulator
    fn daa(&mut self) {
        let mut inc_a: u8 = 0;
        let mut c = self.registers.flags.c;
        let lsb = self.registers.a & 0x0F;
        if (lsb > 9) || self.registers.flags.h {
            inc_a += 0x06;
        }

        let msb = self.registers.a >> 4;
        if (msb > 9) || self.registers.flags.c || (msb >= 9 && lsb > 9) {
            inc_a += 0x60;
            c = true;
        }

        self.add(inc_a);
        self.registers.flags.c = c;
        self.registers.flags.z = self.registers.a == 0x00;
        self.registers.flags.s = (self.registers.a as i8) < 0;
        self.registers.flags.p = self.registers.a.count_ones() & 0x01 == 0x00;
    }

    // NEG
    fn neg(&mut self) {
        let t = !self.registers.a;
        let r = t.wrapping_add(1);
        self.registers.flags.p = self.registers.a == 0x80;
        self.registers.flags.c = self.registers.a != 0;
        self.registers.flags.z = r == 0x00;
        self.registers.flags.s = (r as i8) < 0;
        self.registers.flags.h = (r & 0x0f) != 0x0f;
        self.registers.flags.n = true;
        self.registers.a = r;
    }

    // 16 bits add
    fn add_16(&mut self, n1: u16, n2: u16) -> u16 {
        let r = n1.wrapping_add(n2);
        self.registers.flags.c = u32::from(n1) + u32::from(n2) > 0xffff;
        self.registers.flags.h = (n1 & 0x0800) + (n2 & 0x0800) > 0x0800;
        self.registers.flags.n = false;
        r
    }

    // Register pair addition with carry
    fn addc_16(&mut self, n: u16) {
        let c: u16 = match self.registers.flags.c {
            false => 0,
            true => 1,
        };
        let h = self.registers.get_hl();
        let r = h.wrapping_add(n).wrapping_add(c);
        self.registers.set_hl(r);
        self.registers.flags.s = (r as i16) < 0;
        self.registers.flags.z = r == 0x00;
        self.registers.flags.c = u32::from(h) + u32::from(n) > 0xffff;
        self.registers.flags.h = (h & 0x0800) + (n & 0x0800) > 0x0800;
        self.registers.flags.n = false;
        self.registers.flags.p = {
            let r = ((h as i16).overflowing_add(n as i16)).0.overflowing_add(c as i16);
            r.1
        }
    }

    // Register pair substraction with carry
    fn subc_16(&mut self, n: u16)  {
        let c: u16 = match self.registers.flags.c {
            false => 0,
            true => 1,
        };
        let h = self.registers.get_hl();
        let r = h.wrapping_sub(n).wrapping_sub(c);
        self.registers.set_hl(r);
        self.registers.flags.z = r == 0x00;
        self.registers.flags.s = (r as i16) < 0;
        self.registers.flags.h = (h as i16 & 0x0fff) - (n as i16 & 0x0fff)  - (c as i16) >= 0x00;
        self.registers.flags.c = u16::from(h) < u16::from(n) + u16::from(c);
        self.registers.flags.n = true;
        self.registers.flags.p = {
            let r = ((h as i16).overflowing_sub(n as i16)).0.overflowing_sub(c as i16);
            r.1
        }
    }

    // Rotate accumulator left
    fn rlca(&mut self) {
        self.registers.flags.c = get_bit(self.registers.a, 7);
        self.registers.a = (self.registers.a << 1) | u8::from(self.registers.flags.c);
        self.registers.flags.h = false;
        self.registers.flags.n = false;
    }

    // Rotate accumulator right
    fn rrca(&mut self) {
        self.registers.flags.c = get_bit(self.registers.a, 0);
        self.registers.a = if self.registers.flags.c {0x80 | (self.registers.a >> 1) } else { self.registers.a >> 1 };
        self.registers.flags.h = false;
        self.registers.flags.n = false;
    }

    // RAL Rotate accumulator left through carry
    fn rla(&mut self) {
        let c = self.registers.flags.c;
        self.registers.flags.c = get_bit(self.registers.a, 7);
        self.registers.flags.h = false;
        self.registers.flags.n = false;
        self.registers.a = match c {
            true => (self.registers.a << 1) | 0x01,
            false => self.registers.a << 1
        }
    }
    
    // RAR Rotate accumulator right through carry
    fn rra(&mut self) {
        let c = self.registers.flags.c;
        self.registers.flags.c = get_bit(self.registers.a, 0);
        self.registers.flags.h = false;
        self.registers.flags.n = false;
        self.registers.a = match c {
            true => (self.registers.a >> 1) | 0x80,
            false => self.registers.a >> 1
        }
    }

    pub fn execute(&mut self) -> u32 {
        if self.halt { return 0 };
        let opcode = self.bus.read_byte(self.pc);

        match opcode {
            0xDD | 0xFD | 0xED => {
                let opcode_prefixed = self.bus.read_le_word(self.pc);
                return self.execute_prefixed(opcode_prefixed)
                },
            _ => return self.execute_unprefixed(opcode),
        }
    }

    fn execute_prefixed(&mut self, opcode: u16) -> u32 {
        let cycles = match opcode & 0xFF00 {
                0xDD00 => CYCLES_DD[(opcode & 0x00FF) as usize].into(),
                0xFD00 => CYCLES_FD[(opcode & 0x00FF) as usize].into(),
                0xED00 => CYCLES_ED[(opcode & 0x00FF) as usize].into(),
                _ => 0
        };

        match opcode {
            // 8-Bit Load Group
            // LD r,(IX+d)
            0xDD46 => {                                                             // LD B,(IX+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.b = self.bus.read_byte(self.ix - ( displacement as u16 )) }
                else { self.registers.b = self.bus.read_byte(self.ix + ( displacement as u16 )) }
            },
            0xDD4E => {                                                             // LD C,(IX+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.c = self.bus.read_byte(self.ix - ( displacement as u16 )) }
                else { self.registers.c = self.bus.read_byte(self.ix + ( displacement as u16 )) }
            },
            0xDD56 => {                                                             // LD D,(IX+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.d = self.bus.read_byte(self.ix - ( displacement as u16 )) }
                else { self.registers.d = self.bus.read_byte(self.ix + ( displacement as u16 )) }
            },
            0xDD5E => {                                                             // LD E,(IX+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.e = self.bus.read_byte(self.ix - ( displacement as u16 )) }
                else { self.registers.e = self.bus.read_byte(self.ix + ( displacement as u16 )) }
            },
            0xDD66 => {                                                             // LD H,(IX+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.h = self.bus.read_byte(self.ix - ( displacement as u16 )) }
                else { self.registers.h = self.bus.read_byte(self.ix + ( displacement as u16 )) }
            },
            0xDD6E => {                                                             // LD L,(IX+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.l = self.bus.read_byte(self.ix - ( displacement as u16 )) }
                else { self.registers.l = self.bus.read_byte(self.ix + ( displacement as u16 )) }
            },
            0xDD7E => {                                                             // LD A,(IX+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.a = self.bus.read_byte(self.ix - ( displacement as u16 )) }
                else { self.registers.a = self.bus.read_byte(self.ix + ( displacement as u16 )) }
            },

            // LD r,(IY+d)
            0xFD46 => {                                                             // LD B,(IY+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.b = self.bus.read_byte(self.iy - ( displacement as u16 )) }
                else { self.registers.b = self.bus.read_byte(self.iy + ( displacement as u16 )) }
            },
            0xFD4E => {                                                             // LD C,(IY+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.c = self.bus.read_byte(self.iy - ( displacement as u16 )) }
                else { self.registers.c = self.bus.read_byte(self.iy + ( displacement as u16 )) }
            },
            0xFD56 => {                                                             // LD D,(IY+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.d = self.bus.read_byte(self.iy - ( displacement as u16 )) }
                else { self.registers.d = self.bus.read_byte(self.iy + ( displacement as u16 )) }
            },
            0xFD5E => {                                                             // LD E,(IY+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.e = self.bus.read_byte(self.iy - ( displacement as u16 )) }
                else { self.registers.e = self.bus.read_byte(self.iy + ( displacement as u16 )) }
            },
            0xFD66 => {                                                             // LD H,(IY+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.h = self.bus.read_byte(self.iy - ( displacement as u16 )) }
                else { self.registers.h = self.bus.read_byte(self.iy + ( displacement as u16 )) }
            },
            0xFD6E => {                                                             // LD L,(IY+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.l = self.bus.read_byte(self.iy - ( displacement as u16 )) }
                else { self.registers.l = self.bus.read_byte(self.iy + ( displacement as u16 )) }
            },
            0xFD7E => {                                                             // LD A,(IY+d)
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.registers.a = self.bus.read_byte(self.iy - ( displacement as u16 )) }
                else { self.registers.a = self.bus.read_byte(self.iy + ( displacement as u16 )) }
            },

            // LD (IX+d),r
            0xDD70 => {                                                             // LD (IX+d),B
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.ix - ( displacement as u16 ), self.registers.b) }
                else { self.bus.write_byte(self.ix + ( displacement as u16 ), self.registers.b) }
            },
            0xDD71 => {                                                             // LD (IX+d),C
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.ix - ( displacement as u16 ), self.registers.c) }
                else { self.bus.write_byte(self.ix + ( displacement as u16 ), self.registers.c) }
            },
            0xDD72 => {                                                             // LD (IX+d),D
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.ix - ( displacement as u16 ), self.registers.d) }
                else { self.bus.write_byte(self.ix + ( displacement as u16 ), self.registers.d) }
            },
            0xDD73 => {                                                             // LD (IX+d),E
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.ix - ( displacement as u16 ), self.registers.e) }
                else { self.bus.write_byte(self.ix + ( displacement as u16 ), self.registers.e) }
            },
            0xDD74 => {                                                             // LD (IX+d),H
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.ix - ( displacement as u16 ), self.registers.h) }
                else { self.bus.write_byte(self.ix + ( displacement as u16 ), self.registers.h) }
            },
            0xDD75 => {                                                             // LD (IX+d),L
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.ix - ( displacement as u16 ), self.registers.l) }
                else { self.bus.write_byte(self.ix + ( displacement as u16 ), self.registers.l) }
            },
            0xDD77 => {                                                             // LD (IX+d),A
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.ix - ( displacement as u16 ), self.registers.a) }
                else { self.bus.write_byte(self.ix + ( displacement as u16 ), self.registers.a) }
            },

            // LD (IY+d),r
            0xFD70 => {                                                             // LD (IY+d),B
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.iy - ( displacement as u16 ), self.registers.b) }
                else { self.bus.write_byte(self.iy + ( displacement as u16 ), self.registers.b) }
            },
            0xFD71 => {                                                             // LD (IY+d),C
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.iy - ( displacement as u16 ), self.registers.c) }
                else { self.bus.write_byte(self.iy + ( displacement as u16 ), self.registers.c) }
            },
            0xFD72 => {                                                             // LD (IY+d),D
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.iy - ( displacement as u16 ), self.registers.d) }
                else { self.bus.write_byte(self.iy + ( displacement as u16 ), self.registers.d) }
            },
            0xFD73 => {                                                             // LD (IY+d),E
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.iy - ( displacement as u16 ), self.registers.e) }
                else { self.bus.write_byte(self.iy + ( displacement as u16 ), self.registers.e) }
            },
            0xFD74 => {                                                             // LD (IY+d),H
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.iy - ( displacement as u16 ), self.registers.h) }
                else { self.bus.write_byte(self.iy + ( displacement as u16 ), self.registers.h) }
            },
            0xFD75 => {                                                             // LD (IY+d),L
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.iy - ( displacement as u16 ), self.registers.l) }
                else { self.bus.write_byte(self.iy + ( displacement as u16 ), self.registers.l) }
            },
            0xFD77 => {                                                             // LD (IY+d),A
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 { self.bus.write_byte(self.iy - ( displacement as u16 ), self.registers.a) }
                else { self.bus.write_byte(self.iy + ( displacement as u16 ), self.registers.a) }
            },

            // LD (IX+d),n
            0xDD36 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                let data = self.bus.read_byte(self.pc + 3);
                if displacement < 0 { self.bus.write_byte(self.ix - ( displacement as u16 ), data) }
                else { self.bus.write_byte(self.ix + ( displacement as u16 ), data) }
            }

            // LD IX,nn
            0xDD21 => {
                self.ix = self.bus.read_word(self.pc + 2);
            }


            // LD IY,nn
            0xFD21 => {
                self.iy = self.bus.read_word(self.pc + 2);
            }

            // LD (IY+d),n
            0xFD36 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                let data = self.bus.read_byte(self.pc + 3);
                if displacement < 0 { self.bus.write_byte(self.iy - ( displacement as u16 ), data) }
                else { self.bus.write_byte(self.iy + ( displacement as u16 ), data) }
            }

            // LD A,I
            0xED57 => {
                self.registers.a = self.i;
                self.registers.flags.s = (self.i as i8) < 0;
                self.registers.flags.z = self.i == 0;
                self.registers.flags.h = false;
                self.registers.flags.p = self.iff2;
                self.registers.flags.n = false;
                // TODO :
                // If an interrupt occurs during execution of this instruction, the Parity flag contains a 0.
            },

            // LD A,R
            0xED5F => {
                self.registers.a = self.r;
                self.registers.flags.s = (self.r as i8) < 0;
                self.registers.flags.z = self.r == 0;
                self.registers.flags.h = false;
                self.registers.flags.p = self.iff2;
                self.registers.flags.n = false;
                // TODO :
                // If an interrupt occurs during execution of this instruction, the Parity flag contains a 0.
            },

            // LD I,A
            0xED47 => self.i = self.registers.a,

            // LD R,A
            0xED4F => self.r = self.registers.a,

            // 16-Bit Load Group
            // LD dd,(nn)
            0xED4B => {                                                             // LD BC,(nn)
                let addr = self.bus.read_word(self.pc +2);
                let d = self.bus.read_word(addr);
                self.registers.set_bc(d);
            },

            0xED5B => {                                                             // LD DE,(nn)
                let addr = self.bus.read_word(self.pc +2);
                let d = self.bus.read_word(addr);
                self.registers.set_de(d);
            },

            0xED6B => {                                                             // LD HL,(nn)
                let addr = self.bus.read_word(self.pc +2);
                let d = self.bus.read_word(addr);
                self.registers.set_hl(d);
            },

            0xED7B => {                                                             // LD SP,(nn)
                let addr = self.bus.read_word(self.pc +2);
                let d = self.bus.read_word(addr);
                self.sp = d;
            },

            // LD IX,(nn)
            0xDD2A => {
                let addr = self.bus.read_word(self.pc +2);
                let d = self.bus.read_word(addr);
                self.ix = d;
            },

            // LD IY,(nn)
            0xFD2A => {
                let addr = self.bus.read_word(self.pc +2);
                let d = self.bus.read_word(addr);
                self.iy = d;
            },

            // LD (nn),dd
            0xED43 => {                                                             // LD (nn),BC
                let addr = self.bus.read_word(self.pc +2);
                self.bus.write_word(addr, self.registers.get_bc());
            },

            0xED53 => {                                                             // LD (nn),DE
                let addr = self.bus.read_word(self.pc +2);
                self.bus.write_word(addr, self.registers.get_de());
            },

            0xED63 => {                                                             // LD (nn),HL
                let addr = self.bus.read_word(self.pc +2);
                self.bus.write_word(addr, self.registers.get_hl());
            },

            0xED73 => {                                                             // LD (nn),SP
                let addr = self.bus.read_word(self.pc +2);
                self.bus.write_word(addr, self.sp);
            },

            // LD (nn),IX
            0xDD22 => {
                let addr = self.bus.read_word(self.pc +2);
                self.bus.write_word(addr, self.ix);
            },

            // LD (nn),IY
            0xFD22 => {
                let addr = self.bus.read_word(self.pc +2);
                self.bus.write_word(addr, self.iy);
            },

            // LD SP,IX
            0xDDF9 => self.sp = self.ix,

            // LD SP,IY
            0xFDF9 => self.sp = self.iy,

            // PUSH IX
            0xDDE5 => {
                self.sp = self.sp.wrapping_sub(2);
                self.bus.write_word(self.sp, self.ix);
            },

            // PUSH IY
            0xFDE5 => {
                self.sp = self.sp.wrapping_sub(2);
                self.bus.write_word(self.sp, self.ix);
            },

            // POP IX
            0xDDE1 => {
                self.ix = self.bus.read_word(self.sp);
                self.sp = self.sp.wrapping_add(2);
            },

            // POP IX
            0xFDE1 => {
                self.iy = self.bus.read_word(self.sp);
                self.sp = self.sp.wrapping_add(2);
            },

            // Exchange, Block Transfer, and Search Group
            // EX (SP),IX
            0xDDE3 => {
                let pointed_by_sp = self.bus.read_word(self.sp);
                self.bus.write_word(self.sp, self.ix);
                self.ix = pointed_by_sp;
            },

            // EX (SP),IY
            0xFDE3 => {
                let pointed_by_sp = self.bus.read_word(self.sp);
                self.bus.write_word(self.sp, self.ix);
                self.iy = pointed_by_sp;
            },

            // LDI
            0xEDA0 => {
                self.ldi();
                self.registers.flags.h = false;
                let bc = self.registers.get_bc();
                self.registers.flags.p = bc != 0;
                self.registers.flags.n = false;
            },

            // LDIR
            0xEDB0 => {
                // TODO : When the BC is set to 0 prior to instruction execution, the instruction loops through 64 KB.
                while self.registers.get_bc() !=0 {
                    self.ldi();
                    let bc = self.registers.get_bc();
                    self.registers.flags.h = false;
                    self.registers.flags.p = bc != 0;
                    self.registers.flags.n = false;
                    // TODO : return cycles * number of executions
                }
            },

            // LDD
            0xEDA8 => {
                self.ldd();
                self.registers.flags.h = false;
                let bc = self.registers.get_bc();
                self.registers.flags.p = bc != 0;
                self.registers.flags.n = false;
            },

            // LDDR
            0xEDB8 => {
                // TODO : When the BC is set to 0 prior to instruction execution, the instruction loops through 64 KB.
                while self.registers.get_bc() !=0 {
                    self.ldd();
                    let bc = self.registers.get_bc();
                    self.registers.flags.h = false;
                    self.registers.flags.p = bc != 0;
                    self.registers.flags.n = false;
                    // TODO : return cycles * number of executions
                }
            },

            // CPI
            0xEDA1 => {
                let r = self.cpi();
                self.registers.flags.s = (r as i8) < 0;
                self.registers.flags.z = (r as i8) == 0;
                self.registers.flags.h = (r & 0x0f) != 0x0f;
                self.registers.flags.p = self.registers.get_bc() != 0;
                self.registers.flags.n = true;
            },

            // CPIR
            0xEDB1 => {
                // TODO : When the BC is set to 0 prior to instruction execution, the instruction loops through 64 KB.
                while self.registers.get_bc() !=0 {
                    let r = self.cpi();
                    self.registers.flags.s = (r as i8) < 0;
                    self.registers.flags.z = (r as i8) == 0;
                    self.registers.flags.h = (r & 0x0f) != 0x0f;
                    self.registers.flags.p = self.registers.get_bc() != 0;
                    self.registers.flags.n = true;
                    if self.registers.flags.z { break }
                    // TODO : return cycles * number of executions
                }
            },

            // CPD
            0xEDA9 => {
                let r = self.cpd();
                self.registers.flags.s = (r as i8) < 0;
                self.registers.flags.z = (r as i8) == 0;
                self.registers.flags.h = (r & 0x0f) != 0x0f;
                self.registers.flags.p = self.registers.get_bc() != 0;
                self.registers.flags.n = true;
            },

            // CPDR
            0xEDB9 => {
                while self.registers.get_bc() !=0 {
                    let r = self.cpd();
                    self.registers.flags.s = (r as i8) < 0;
                    self.registers.flags.z = (r as i8) == 0;
                    self.registers.flags.h = (r & 0x0f) != 0x0f;
                    self.registers.flags.p = self.registers.get_bc() != 0;
                    self.registers.flags.n = true;
                    if self.registers.flags.z { break }
                    // TODO : return cycles * number of executions
                }
            },

            // 8-Bit Arithmetic Group
            // ADD A,(IX+d)
            0xDD86 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.ix - ( displacement as u16 ));
                    self.add(d);
                }
                else {
                    let d = self.bus.read_byte(self.ix + ( displacement as u16 ));
                    self.add(d);
                }
            },

            // ADD A,(IX+d)
            0xFD86 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.iy - ( displacement as u16 ));
                    self.add(d);
                }
                else {
                    let d = self.bus.read_byte(self.iy + ( displacement as u16 ));
                    self.add(d);
                }
            },

            // ADD A,(IX+d)
            0xDD8E => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.ix - ( displacement as u16 ));
                    self.add(d);
                }
                else {
                    let d = self.bus.read_byte(self.ix + ( displacement as u16 ));
                    self.addc(d);
                }
            },

            // ADD A,(IY+d)
            0xFD8E => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.iy - ( displacement as u16 ));
                    self.add(d);
                }
                else {
                    let d = self.bus.read_byte(self.iy + ( displacement as u16 ));
                    self.addc(d);
                }
            },

            // SUB (IX+d)
            0xDD96 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.ix - ( displacement as u16 ));
                    self.sub(d);
                }
                else {
                    let d = self.bus.read_byte(self.ix + ( displacement as u16 ));
                    self.sub(d);
                }
            },

            // SUB (IX+d)
            0xFD96 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.iy - ( displacement as u16 ));
                    self.sub(d);
                }
                else {
                    let d = self.bus.read_byte(self.iy + ( displacement as u16 ));
                    self.sub(d);
                }
            },

            // SUB (IX+d)
            0xDD9E => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.ix - ( displacement as u16 ));
                    self.subc(d);
                }
                else {
                    let d = self.bus.read_byte(self.ix + ( displacement as u16 ));
                    self.subc(d);
                }
            },

            // SUB (IY+d)
            0xFD9E => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.iy - ( displacement as u16 ));
                    self.subc(d);
                }
                else {
                    let d = self.bus.read_byte(self.iy + ( displacement as u16 ));
                    self.subc(d);
                }
            },

            // AND (IX+d)
            0xDDA6 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.ix - ( displacement as u16 ));
                    self.and(d);
                }
                else {
                    let d = self.bus.read_byte(self.ix + ( displacement as u16 ));
                    self.and(d);
                }
            },

            // AND (IY+d)
            0xFDA6 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.iy - ( displacement as u16 ));
                    self.and(d);
                }
                else {
                    let d = self.bus.read_byte(self.iy + ( displacement as u16 ));
                    self.and(d);
                }
            },

            // OR (IX+d)
            0xDDB6 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.ix - ( displacement as u16 ));
                    self.or(d);
                }
                else {
                    let d = self.bus.read_byte(self.ix + ( displacement as u16 ));
                    self.or(d);
                }
            },

            // OR (IY+d)
            0xFDB6 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.iy - ( displacement as u16 ));
                    self.or(d);
                }
                else {
                    let d = self.bus.read_byte(self.iy + ( displacement as u16 ));
                    self.or(d);
                }
            },

            // XOR (IX+d)
            0xDDAE => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.ix - ( displacement as u16 ));
                    self.xor(d);
                }
                else {
                    let d = self.bus.read_byte(self.ix + ( displacement as u16 ));
                    self.xor(d);
                }
            },

            // XOR (IY+d)
            0xFDAE => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.iy - ( displacement as u16 ));
                    self.xor(d);
                }
                else {
                    let d = self.bus.read_byte(self.iy + ( displacement as u16 ));
                    self.xor(d);
                }
            },

            // CP (IX+d)
            0xDDBE => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.ix - ( displacement as u16 ));
                    self.cp(d);
                }
                else {
                    let d = self.bus.read_byte(self.ix + ( displacement as u16 ));
                    self.cp(d);
                }
            },

            // CP (IY+d)
            0xFDBE => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let d = self.bus.read_byte(self.iy - ( displacement as u16 ));
                    self.cp(d);
                }
                else {
                    let d = self.bus.read_byte(self.iy + ( displacement as u16 ));
                    self.cp(d);
                }
            },

            // INC (IX+d)
            0xDD34 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let m = self.ix - ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.inc(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m =self.ix + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.inc(d);
                    self.bus.write_byte(m, r);
                }
            },

            // INC (IY+d)
            0xFD34 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let m = self.iy - ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.inc(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.iy + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.inc(d);
                    self.bus.write_byte(m, r);
                }
            },

            // DEC (IX+d)
            0xDD35 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let m = self.ix - ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.dec(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m =self.ix + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.dec(d);
                    self.bus.write_byte(m, r);
                }
            },

            // DEC (IY+d)
            0xFD35 => {
                let displacement: i8 = self.bus.read_byte(self.pc + 2) as i8;
                if displacement < 0 {
                    let m = self.iy - ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.dec(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.iy + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.dec(d);
                    self.bus.write_byte(m, r);
                }
            },

            // General-Purpose Arithmetic and CPU Control Groups
            // NEG
            0xED44 => {
                self.neg();
            },

            // Interrput modes
            0xED46 => self.im = 0,
            0xED56 => self.im = 1,
            0xED5E => self.im = 2,

            //16-Bit Arithmetic Group
            // ADC HL,ss
            0xED4A => {                                                     // ADC HL,BC
                let reg = self.registers.get_bc();
                self.addc_16(reg);
            },
            0xED5A => {                                                     // ADC HL,DE
                let reg = self.registers.get_de();
                self.addc_16(reg);
            },
            0xED6A => {                                                     // ADC HL,HL
                let reg = self.registers.get_hl();
                self.addc_16(reg);
            },
            0xED7A => {                                                     // ADC HL,SP
                let reg = self.sp;
                self.addc_16(reg);
            },

            // SBC HL,ss
            0xED42 => {                                                     // SBC HL,BC
                let reg = self.registers.get_bc();
                self.subc_16(reg);
            },
            0xED52 => {                                                     // SBC HL,DE
                let reg = self.registers.get_de();
                self.subc_16(reg);
            },
            0xED62 => {                                                     // SBC HL,HL
                let reg = self.registers.get_hl();
                self.subc_16(reg);
            },
            0xED72 => {                                                     // SBC HL,SP
                let reg = self.sp;
                self.subc_16(reg);
            },

            // ADD IX,pp
            0xDD09 => {                                                     // ADD IX,BC
                let reg = self.registers.get_bc();
                let r = self.add_16(self.ix, reg);
                self.ix = r;
            },
            0xDD19 => {                                                     // ADD IX,DE
                let reg = self.registers.get_de();
                let r = self.add_16(self.ix, reg);
                self.ix = r;
            },
            0xDD29 => {                                                     // ADD IX,IX
                let reg = self.ix;
                let r = self.add_16(self.ix, reg);
                self.ix = r;
            },
            0xDD39 => {                                                     // ADD IX,SP
                let reg = self.sp;
                let r = self.add_16(self.ix, reg);
                self.ix = r;
            },

            // ADD IY,pp
            0xFD09 => {                                                     // ADD IY,BC
                let reg = self.registers.get_bc();
                let r = self.add_16(self.iy, reg);
                self.iy = r;
            },

            0xFD19 => {                                                     // ADD IY,DE
                let reg = self.registers.get_de();
                let r = self.add_16(self.iy, reg);
                self.iy = r;
            },

            0xFD29 => {                                                     // ADD IY,IY
                let reg = self.ix;
                let r = self.add_16(self.iy, reg);
                self.iy = r;
            },

            0xFD39 => {                                                     // ADD IY,SP
                let reg = self.sp;
                let r = self.add_16(self.iy, reg);
                self.iy = r;
            },

            0xDD23 => {                                                     // INC IX
                let r = self.ix.wrapping_add(1);
                self.ix = r;
            },

            0xFD23 => {                                                     // INC IY
                let r = self.iy.wrapping_add(1);
                self.iy = r;
            },

            0xDD2B => {                                                     // DEC IX
                let r = self.ix.wrapping_sub(1);
                self.ix = r;
            },

            0xFD2B => {                                                     // DEC IY
                let r = self.iy.wrapping_sub(1);
                self.iy = r;
            },

            _ => {}
        }

        match opcode {
            0xED57 | 0xED5F | 0xED47 | 0xED4F | 0xDDF9 | 0xFDF9 |
            0xDDE5 | 0xFDE5 | 0xDDE1 | 0xFDE1 | 0xDDE3 | 0xFDE3 |
            0xEDA0 | 0xEDB0 | 0xEDA8 | 0xEDB8 | 0xEDA1 | 0xEDB1 |
            0xEDA9 | 0xEDB9 | 0xED44 |
            0xED4A | 0xED5A | 0xED6A | 0xED7A |
            0xED42 | 0xED52 | 0xED62 | 0xED72 |
            0xDD09 | 0xDD19 | 0xDD29 | 0xDD39 |
            0xFD09 | 0xFD19 | 0xFD29 | 0xFD39 |
            0xDD23 | 0xFD23 | 0xDD2B | 0xFD2B => self.pc += 2,
            0xDD46 | 0xFD46 | 0xDD4E | 0xFD4E | 0xDD56 | 0xFD56 |
            0xDD5E | 0xFD5E | 0xDD66 | 0xFD66 | 0xDD6E | 0xFD6E |
            0xDD7E | 0xFD7E |
            0xDD70 | 0xDD71 | 0xDD72 | 0xDD73 | 0xDD74 | 0xDD75 |
            0xDD77 |
            0xFD70 | 0xFD71 | 0xFD72 | 0xFD73 | 0xFD74 | 0xFD75 |
            0xFD77 | 0xDD86 | 0xFD86 | 0xDD8E | 0xFD8E |
            0xDD96 | 0xFD96 | 0xDD9E | 0xFD9E | 0xDDA6 | 0xFDA6 |
            0xDDB6 | 0xFDB6 | 0xDDAE | 0xFDAE | 0xDDBE | 0xFDBE |
            0xDD34 | 0xFD34 | 0xDD35 | 0xFD35=> self.pc += 3,
            0xDD36 | 0xFD36 | 0xDD21 | 0xFD21 | 0xED4B | 0xED5B |
            0xED6B | 0xED7B | 0xDD2A | 0xFD2A |
            0xED43 | 0xED53 | 0xED63 | 0xED73 |
            0xDD22 | 0xFD22 => self.pc += 4,
            _ => self.pc +=1,
        }

        cycles
    }

    fn execute_unprefixed(&mut self, opcode: u8) -> u32 {
        let cycles = CYCLES[opcode as usize].into();

        match opcode {
            // 8-Bit Load Group
            // LD r,r'      LD r,(HL)
            0x40 => {},                                                             // LD B,B
            0x41 => self.registers.b = self.registers.c,                            // LD B,C
            0x42 => self.registers.b = self.registers.d,                            // LD B,D
            0x43 => self.registers.b = self.registers.e,                            // LD B,E
            0x44 => self.registers.b = self.registers.h,                            // LD B,H
            0x45 => self.registers.b = self.registers.l,                            // LD B,L
            0x46 => {                                                               // LD B,(HL)
                let addr = self.registers.get_hl();
                self.registers.b = self.bus.read_byte(addr)
            }
            0x47 => self.registers.b = self.registers.a,                            // LD B,A

            0x48 => self.registers.c = self.registers.b,                            // LD C,B
            0x49 => {},                                                             // LD C,C
            0x4A => self.registers.c = self.registers.d,                            // LD C,D
            0x4B => self.registers.c = self.registers.e,                            // LD C,E
            0x4C => self.registers.c = self.registers.h,                            // LD C,H
            0x4D => self.registers.c = self.registers.l,                            // LD C,L
            0x4E => {                                                               // LD C,(HL)
                let addr = self.registers.get_hl();
                self.registers.c = self.bus.read_byte(addr)
            }
            0x4F => self.registers.c = self.registers.a,                            // LD C,A

            0x50 => self.registers.d = self.registers.b,                            // LD D,B
            0x51 => self.registers.d = self.registers.c,                            // LD D,C
            0x52 => {},                                                             // LD D,D
            0x53 => self.registers.d = self.registers.e,                            // LD D,E
            0x54 => self.registers.d = self.registers.h,                            // LD D,H
            0x55 => self.registers.d = self.registers.l,                            // LD D,L
            0x56 => {                                                               // LD D,(HL)
                let addr = self.registers.get_hl();
                self.registers.d = self.bus.read_byte(addr)
            }
            0x57 => self.registers.d = self.registers.a,                            // LD D,A

            0x58 => self.registers.e = self.registers.b,                            // LD E,B
            0x59 => self.registers.e = self.registers.c,                            // LD E,C
            0x5A => self.registers.e = self.registers.d,                            // LD E,D
            0x5B => {},                                                             // LD E,E
            0x5C => self.registers.e = self.registers.h,                            // LD E,H
            0x5D => self.registers.e = self.registers.l,                            // LD E,L
            0x5E => {                                                               // LD E,(HL)
                let addr = self.registers.get_hl();
                self.registers.e = self.bus.read_byte(addr)
            }
            0x5F => self.registers.e = self.registers.a,                            // LD E,A

            0x60 => self.registers.h = self.registers.b,                            // LD H,B
            0x61 => self.registers.h = self.registers.c,                            // LD H,C
            0x62 => self.registers.h = self.registers.d,                            // LD H,D
            0x63 => self.registers.h = self.registers.e,                            // LD H,E
            0x64 => {},                                                             // LD H,H
            0x65 => self.registers.h = self.registers.l,                            // LD H,L
            0x66 => {                                                               // LD H,(HL)
                let addr = self.registers.get_hl();
                self.registers.h = self.bus.read_byte(addr)
            }
            0x67 => self.registers.h = self.registers.a,                            // LD H,A

            0x68 => self.registers.l = self.registers.b,                            // LD L,B
            0x69 => self.registers.l = self.registers.c,                            // LD L,C
            0x6A => self.registers.l = self.registers.d,                            // LD L,D
            0x6B => self.registers.l = self.registers.e,                            // LD L,E
            0x6C => self.registers.l = self.registers.h,                            // LD L,H
            0x6D => {},                                                             // LD L,L
            0x6E => {                                                               // LD L,(HL)
                let addr = self.registers.get_hl();
                self.registers.l = self.bus.read_byte(addr)
            }
            0x6F => self.registers.l = self.registers.a,                            // LD L,A

            0x78 => self.registers.a = self.registers.b,                            // LD A,B
            0x79 => self.registers.a = self.registers.c,                            // LD A,C
            0x7A => self.registers.a = self.registers.d,                            // LD A,D
            0x7B => self.registers.a = self.registers.e,                            // LD A,E
            0x7C => self.registers.a = self.registers.h,                            // LD A,H
            0x7D => self.registers.a = self.registers.l,                            // LD A,L
            0x7E => {                                                               // LD A,(HL)
                let addr = self.registers.get_hl();
                self.registers.a = self.bus.read_byte(addr)
            }
            0x7F => {},                                                             // LD A,A

            // LD (HL),r
            0x70 => {                                                               // LD (HL), B
                let addr = self.registers.get_hl();
                self.bus.write_byte(addr, self.registers.b)
            },
            0x71 => {                                                               // LD (HL), C
                let addr = self.registers.get_hl();
                self.bus.write_byte(addr, self.registers.c)
            },
            0x72 => {                                                               // LD (HL), D
                let addr = self.registers.get_hl();
                self.bus.write_byte(addr, self.registers.d)
            },
            0x73 => {                                                               // LD (HL), E
                let addr = self.registers.get_hl();
                self.bus.write_byte(addr, self.registers.e)
            },
            0x74 => {                                                               // LD (HL), H
                let addr = self.registers.get_hl();
                self.bus.write_byte(addr, self.registers.h)
            },
            0x75 => {                                                               // LD (HL), L
                let addr = self.registers.get_hl();
                self.bus.write_byte(addr, self.registers.l)
            },

            0x77 => {                                                               // LD (HL), A
                let addr = self.registers.get_hl();
                self.bus.write_byte(addr, self.registers.a)
            },

            // LD r,n
            0x06 => {                                                               // LD B,n
                let data = self.bus.read_byte(self.pc + 1);
                self.registers.b = data;
            },
            0x0E => {                                                               // LD C,n
                let data = self.bus.read_byte(self.pc + 1);
                self.registers.c = data;
            },
            0x16 => {                                                               // LD D,n
                let data = self.bus.read_byte(self.pc + 1);
                self.registers.d = data;
            },
            0x1E => {                                                               // LD E,n
                let data = self.bus.read_byte(self.pc + 1);
                self.registers.e = data;
            },
            0x26 => {                                                               // LD H,n
                let data = self.bus.read_byte(self.pc + 1);
                self.registers.h = data;
            },
            0x2E => {                                                               // LD L,n
                let data = self.bus.read_byte(self.pc + 1);
                self.registers.l = data;
            },
            0x36 => {                                                               // LD (HL),n
                let data = self.bus.read_byte(self.pc + 1);
                let addr = self.registers.get_hl();
                self.bus.write_byte(addr, data);
            },
            0x3E => {                                                               // LD A,n
                let data = self.bus.read_byte(self.pc + 1);
                self.registers.a = data;
            },

            // LD A,(BC)
            0x0A => {
                let addr = self.registers.get_bc();
                self.registers.a = self.bus.read_byte(addr)
            },

            // LD A,(DE)
            0x1A => {
                let addr = self.registers.get_de();
                self.registers.a = self.bus.read_byte(addr)
            },

            // LD (nn),A
            0x32 => {
                let addr = self.bus.read_word(self.pc + 1);
                self.bus.write_byte(addr, self.registers.a);
            },

            // 16-Bit Load Group
            // LD dd,nn
            0x01 => {                                                               // LD BC,nn
                let d16 = self.bus.read_word(self.pc + 1); 
                self.registers.set_bc(d16);
            },
            0x11 => {                                                               // LD DE,nn
                let d16 = self.bus.read_word(self.pc + 1); 
                self.registers.set_de(d16);
            },
            0x21 => {                                                               // LD HL,nn
                let d16 = self.bus.read_word(self.pc + 1); 
                self.registers.set_hl(d16);
            },
            0x31 => {                                                               // LD SP,nn
                let d16 = self.bus.read_word(self.pc + 1); 
                self.sp = d16;
            },

            // LD HL,(nn)
            0x2A => {
                let addr = self.bus.read_word(self.pc + 1);
                let d = self.bus.read_word(addr);
                self.registers.set_hl(d);
            },

            // LD (nn),HL
            0x22 => {
                let d = self.registers.get_hl();
                let addr = self.bus.read_word(self.pc + 1);
                self.bus.write_word(addr, d);
            },

            // LD SP,HL
            0xF9 => self.sp = self.registers.get_hl(),

            // PUSH qq
            0xC5 => {                                                               // PUSH BC
                self.sp = self.sp.wrapping_sub(2);
                self.bus.write_word(self.sp, self.registers.get_bc());
            },
            0xD5 => {                                                               // PUSH DE
                self.sp = self.sp.wrapping_sub(2);
                self.bus.write_word(self.sp, self.registers.get_de());
            },
            0xE5 => {                                                               // PUSH HL
                self.sp = self.sp.wrapping_sub(2);
                self.bus.write_word(self.sp, self.registers.get_hl());
            },
            0xF5 => {                                                               // PUSH AF
                self.sp = self.sp.wrapping_sub(2);
                self.bus.write_byte(self.sp, self.registers.flags.to_byte());
                self.bus.write_byte(self.sp + 1, self.registers.a);
            },

            // POP qq
            0xC1 => {                                                               // POP BC
                self.registers.set_bc(self.bus.read_word(self.sp));
                self.sp = self.sp.wrapping_add(2);
            },

            0xD1 => {                                                               // POP DE
                self.registers.set_de(self.bus.read_word(self.sp));
                self.sp = self.sp.wrapping_add(2);
            },

            0xE1 => {                                                               // POP HL
                self.registers.set_hl(self.bus.read_word(self.sp));
                self.sp = self.sp.wrapping_add(2);
            },

            0xF1 => {                                                               // POP AF
                self.registers.a = self.bus.read_byte((self.sp)+1);
                let bflags = self.bus.read_byte(self.sp);
                self.registers.flags.from_byte(bflags);
                self.sp = self.sp.wrapping_add(2);
            },

            // Exchange, Block Transfer, and Search Group
            // EX DE,HL
            0xEB => {
                let de= self.registers.get_de();
                let hl = self.registers.get_hl();
                self.registers.set_de(hl);
                self.registers.set_hl(de);
            }

            // EX AF,AF'
            0x08 => {
                let af = self.registers.get_af();
                let afp = self.alt_registers.get_af();
                self.registers.set_af(afp);
                self.alt_registers.set_af(af);
            }

            // EXX
            0xD9 => {
                let bc = self.registers.get_bc();
                let de = self.registers.get_de();
                let hl = self.registers.get_hl();
                let bcp = self.alt_registers.get_bc();
                let dep = self.alt_registers.get_de();
                let hlp = self.alt_registers.get_hl();
                self.registers.set_bc(bcp);
                self.registers.set_de(dep);
                self.registers.set_hl(hlp);
                self.alt_registers.set_bc(bc);
                self.alt_registers.set_de(de);
                self.alt_registers.set_hl(hl);
            }

            // EX (SP),HL
            0xE3 => {
                let pointed_by_sp = self.bus.read_word(self.sp);
                let hl = self.registers.get_hl();
                self.bus.write_word(self.sp, hl);
                self.registers.set_hl(pointed_by_sp);
            },

            // 8-Bit Arithmetic Group
            // ADD A,r
            0x80 => self.add(self.registers.b),                                   // ADD A,B
            0x81 => self.add(self.registers.c),                                   // ADD C
            0x82 => self.add(self.registers.d),                                   // ADD D
            0x83 => self.add(self.registers.e),                                   // ADD E
            0x84 => self.add(self.registers.h),                                   // ADD H
            0x85 => self.add(self.registers.l),                                   // ADD L
            0x86 => {                                                              // ADD (HL)
                let addr = self.registers.get_hl();
                let n = self.bus.read_byte(addr);
                self.add(n)
            },
            0x87 => self.add(self.registers.a),                                    // ADD A

            // ADD A,n
            0xC6 => {
                let n = self.bus.read_byte(self.pc + 1);
                self.add(n);
            },

            // ADC A,r
            0x88 => self.addc(self.registers.b),                                    // ADC A,B
            0x89 => self.addc(self.registers.c),                                    // ADC A,C
            0x8A => self.addc(self.registers.d),                                    // ADC A,D
            0x8B => self.addc(self.registers.e),                                    // ADC A,E
            0x8C => self.addc(self.registers.h),                                    // ADC A,H
            0x8D => self.addc(self.registers.l),                                    // ADC A,L
            0x8E => {                                                               // ADC A,(HL)
                let addr = self.registers.get_hl();
                let n = self.bus.read_byte(addr);
                self.addc(n)
            },
            0x8F => self.addc(self.registers.a),                                    // ADC A,A

            // ADC a,n
            0xCE => {                                                               // ADC A,(HL)
                let n = self.bus.read_byte(self.pc + 1);
                self.addc(n)
            },

            // SUB s
            0x90 => self.sub(self.registers.b),                                     // SUB B
            0x91 => self.sub(self.registers.c),                                     // SUB C
            0x92 => self.sub(self.registers.d),                                     // SUB D
            0x93 => self.sub(self.registers.e),                                     // SUB E
            0x94 => self.sub(self.registers.h),                                     // SUB H
            0x95 => self.sub(self.registers.l),                                     // SUB L
            0x96 => {                                                               // SUB (HL)
                let addr = self.registers.get_hl();
                let n = self.bus.read_byte(addr);
                self.sub(n)
            },
            0x97 => self.sub(self.registers.a),                                     // SUB A

            0xD6 => {                                                               // SUB n
                let n = self.bus.read_byte(self.pc + 1);
                self.sub(n);
            },

            // SBC A,s
            0x98 => self.subc(self.registers.b),                                    // SBC A,B
            0x99 => self.subc(self.registers.c),                                    // SBC A,C
            0x9A => self.subc(self.registers.d),                                    // SBC A,D
            0x9B => self.subc(self.registers.e),                                    // SBC A,E
            0x9C => self.subc(self.registers.h),                                    // SBC A,H
            0x9D => self.subc(self.registers.l),                                    // SBC A,L
            0x9E => {                                                               // SBC A,(HL)
                let addr = self.registers.get_hl();
                let n = self.bus.read_byte(addr);
                self.subc(n)
            },
            0x9F => self.subc(self.registers.a),                                    // SBC A,A

            0xDE => {                                                               // SBC A,n
                let n = self.bus.read_byte(self.pc + 1);
                self.subc(n);
            },

            // AND s
            0xA0 => self.and(self.registers.b),                                     // AND B
            0xA1 => self.and(self.registers.c),                                     // AND C
            0xA2 => self.and(self.registers.d),                                     // AND D
            0xA3 => self.and(self.registers.e),                                     // AND E
            0xA4 => self.and(self.registers.h),                                     // AND H
            0xA5 => self.and(self.registers.l),                                     // AND L
            0xA6 => {                                                               // AND (HL)
                let addr = self.registers.get_hl();
                let n = self.bus.read_byte(addr);
                self.and(n)
            },
            0xA7 => self.and(self.registers.a),                                     // AND A

            0xE6 => {                                                               // AND n
                let n = self.bus.read_byte(self.pc + 1);
                self.and(n);
            },

            // OR s
            0xB0 => self.or(self.registers.b),                                      // OR B
            0xB1 => self.or(self.registers.c),                                      // OR C
            0xB2 => self.or(self.registers.d),                                      // OR D
            0xB3 => self.or(self.registers.e),                                      // OR E
            0xB4 => self.or(self.registers.h),                                      // OR H
            0xB5 => self.or(self.registers.l),                                      // OR L
            0xB6 => {                                                               // OR (HL)
                let addr = self.registers.get_hl();
                let n = self.bus.read_byte(addr);
                self.or(n)
            },
            0xB7 => self.or(self.registers.a),                                      // OR A

            0xF6 => {                                                               // OR n
                let n = self.bus.read_byte(self.pc + 1);
                self.or(n);
            },

            // XOR s
            0xA8 => self.xor(self.registers.b),                                     // XOR B
            0xA9 => self.xor(self.registers.c),                                     // XOR C
            0xAA => self.xor(self.registers.d),                                     // XOR D
            0xAB => self.xor(self.registers.e),                                     // XOR E
            0xAC => self.xor(self.registers.h),                                     // XOR H
            0xAD => self.xor(self.registers.l),                                     // XOR L
            0xAE => {                                                               // XOR (HL)
                let addr = self.registers.get_hl();
                let n = self.bus.read_byte(addr);
                self.xor(n)
            },
            0xAF => self.xor(self.registers.a),                                     // XOR A

            0xEE => {                                                               // XOR n
                let n = self.bus.read_byte(self.pc + 1);
                self.xor(n);
            },

            // CMP s
            0xB8 => self.cp(self.registers.b),                                      // CP B
            0xB9 => self.cp(self.registers.c),                                      // CP C
            0xBA => self.cp(self.registers.d),                                      // CP D
            0xBB => self.cp(self.registers.e),                                      // CP E
            0xBC => self.cp(self.registers.h),                                      // CP H
            0xBD => self.cp(self.registers.l),                                      // CP L
            0xBE => {                                                               // CP (HL)
                let addr = self.registers.get_hl();
                let n = self.bus.read_byte(addr);
                self.cp(n)
            },
            0xBF => self.cp(self.registers.a),                                      // CP A

            0xFE => {                                                               // CP n
                let n = self.bus.read_byte(self.pc + 1);
                self.cp(n);
            },

            // INC r
            0x04 => self.registers.b = self.inc(self.registers.b),                  // INC B
            0x0C => self.registers.c = self.inc(self.registers.c),                  // INC C
            0x14 => self.registers.d = self.inc(self.registers.d),                  // INC D
            0x1C => self.registers.e = self.inc(self.registers.e),                  // INC E
            0x24 => self.registers.h = self.inc(self.registers.h),                  // INC H
            0x2C => self.registers.l = self.inc(self.registers.l),                  // INC L
            0x34 => {                                                               // INC (HL)
                let addr = self.registers.get_hl();
                let r = self.inc(self.bus.read_byte(addr));
                self.bus.write_byte(addr, r);
            },
            0x3C => self.registers.a = self.inc(self.registers.a),                  // INC A

            // DEC m
            0x05 => self.registers.b = self.dec(self.registers.b),                  // DEC B
            0x0D => self.registers.c = self.dec(self.registers.c),                  // DEC C
            0x15 => self.registers.d = self.dec(self.registers.d),                  // DEC D
            0x1D => self.registers.e = self.dec(self.registers.e),                  // DEC E
            0x25 => self.registers.h = self.dec(self.registers.h),                  // DEC H
            0x2D => self.registers.l = self.dec(self.registers.l),                  // DEC L
            0x35 => {                                                               // DEC (HL)
                let addr = self.registers.get_hl();
                let r = self.dec(self.bus.read_byte(addr));
                self.bus.write_byte(addr, r);
            },
            0x3D => self.registers.a = self.dec(self.registers.a),                  // DEC A

            // General-Purpose Arithmetic and CPU Control Groups
            // DAA
            0x27 => self.daa(),

            // CPL
            0x2F => {
                self.registers.a = !self.registers.a;
                self.registers.flags.h = true;
                self.registers.flags.n = true;
            },

            // CCF
            0x3F => {
                self.registers.flags.h = self.registers.flags.c;
                self.registers.flags.c = !self.registers.flags.c;
                self.registers.flags.n = false;
            },

            // SCF
            0x37 => {
                self.registers.flags.c = true;
                self.registers.flags.h = false;
                self.registers.flags.n = false;
            },

            // NOP
            0x00 => {},

            // HALT
            0x76 => self.halt = true,

            // DI
            0xF3 => {
                self.iff1 = false;
                self.iff2 = false;
            },

            // EI
            0xFB => {
                self.iff1 = true;
                self.iff2 = true;
            },

            // 16-Bit Arithmetic Group
            // ADD HL,ss
            0x09 => {                                                       // ADD HL,BC
                let reg = self.registers.get_bc();
                let r = self.add_16(self.registers.get_hl(), reg);
                self.registers.set_hl(r);
            },
            0x19 => {                                                       // ADD HL,DE
                let reg = self.registers.get_de();
                let r = self.add_16(self.registers.get_hl(), reg);
                self.registers.set_hl(r);
            },
            0x29 => {                                                       // ADD HL,HL
                let reg = self.registers.get_hl();
                let r = self.add_16(self.registers.get_hl(), reg);
                self.registers.set_hl(r);
            },
            0x39 => {                                                       // ADD HL,SP
                let reg = self.sp;
                let r = self.add_16(self.registers.get_hl(), reg);
                self.registers.set_hl(r);
            },

            // INC ss
            0x03 => {                                                       // INC BC
                let r = self.registers.get_bc().wrapping_add(1);
                self.registers.set_bc(r);
            },

            0x13 => {                                                       // INC DE
                let r = self.registers.get_de().wrapping_add(1);
                self.registers.set_de(r);
            },

            0x23 => {                                                       // INC HL
                let r = self.registers.get_hl().wrapping_add(1);
                self.registers.set_hl(r);
            },

            0x33 => {                                                       // INC SP
                let r = self.sp.wrapping_add(1);
                self.sp = r;
            },

            // DEC ss
            0x0B => {                                                       // DEC BC
                let r = self.registers.get_bc().wrapping_sub(1);
                self.registers.set_bc(r);
            },

            0x1B => {                                                       // DEC DE
                let r = self.registers.get_de().wrapping_sub(1);
                self.registers.set_de(r);
            },

            0x2B => {                                                       // DEC HL
                let r = self.registers.get_hl().wrapping_sub(1);
                self.registers.set_hl(r);
            },

            0x3B => {                                                       // DEC SP
                let r = self.sp.wrapping_sub(1);
                self.sp = r;
            },

            // Rotate and Shift Group
            // RLCA
            0x07 => self.rlca(),

            // RLA
            0x17 => self.rla(),

            // RRCA
            0x0F => self.rrca(),

            // RRA
            0x1F => self.rra(),

            _ => {},
        }

        match opcode {
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E |
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6  | 0xF6 | 0xEE | 0xFE => self.pc += 2,
            0x32 | 0x01 | 0x11 | 0x21 | 0x31 | 0x2A | 0x22 => self.pc += 3,
            _ => self.pc +=1,
        }

        cycles
    }
}