use crate::registers::Registers;
use crate::memory::AddressBus;

const CYCLES: [u8; 256] = [
    0, 10, 0, 0, 0, 0, 7, 0, 4, 0, 7, 0, 0, 0, 7, 0,
    0, 10, 0, 0, 0, 0, 7, 0, 0, 0, 7, 0, 0, 0, 7, 0,
    0, 10, 16, 0, 0, 0, 7, 0, 0, 0, 16, 0, 0, 0, 7, 0,
    0, 10, 13, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    7, 7, 7, 7, 7, 7, 4, 7, 4, 4, 4, 4, 4, 4, 7, 4,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 10, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 10, 0, 0, 0, 11, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0,
    0, 10, 0, 19, 0, 11, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0,
    0, 10, 0, 0, 0, 11, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0,
];

const CYCLES_DD: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 14, 20, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    19, 19, 19, 19, 19, 19, 0, 19, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 14, 0, 23, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0,
];

const CYCLES_FD: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 14, 20, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 19, 0,
    19, 19, 19, 19, 19, 19, 0, 19, 0, 0, 0, 0, 0, 0, 19, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
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
    0, 0, 0, 20, 0, 0, 0, 9, 0, 0, 0, 20, 0, 0, 0, 9,
    0, 0, 0, 20, 0, 0, 0, 9, 0, 0, 0, 20, 0, 0, 0, 9,
    0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0,
    0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    16, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0,
    21, 0, 0, 0, 0, 0, 0, 0, 21, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

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
    pub halt: bool,
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
            halt: false,
            iff2: false,
        }
    }

    fn ldi(&mut self) {
        let bc = self.registers.get_bc();
        let de = self.registers.get_de();
        let hl = self.registers.get_hl();
        self.bus.write_byte(de, self.bus.read_byte(hl));
        self.registers.set_de(de.saturating_add(1));
        self.registers.set_hl(hl.saturating_add(1));
        self.registers.set_bc(bc.saturating_sub(1));
    }

    fn ldd(&mut self) {
        let bc = self.registers.get_bc();
        let de = self.registers.get_de();
        let hl = self.registers.get_hl();
        self.bus.write_byte(de, self.bus.read_byte(hl));
        self.registers.set_de(de.saturating_sub(1));
        self.registers.set_hl(hl.saturating_sub(1));
        self.registers.set_bc(bc.saturating_sub(1));
    }

    pub fn execute(&mut self) -> u32 {
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
                while self.registers.get_bc() !=0 {
                self.ldi();
                let bc = self.registers.get_bc();
                self.registers.flags.h = false;
                self.registers.flags.p = bc != 0;
                self.registers.flags.n = false;
                // TODO : return 16 cycles if BC = 0
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
                while self.registers.get_bc() !=0 {
                self.ldd();
                let bc = self.registers.get_bc();
                self.registers.flags.h = false;
                self.registers.flags.p = bc != 0;
                self.registers.flags.n = false;
                // TODO : return 16 cycles if BC = 0
                }
            },

            // CPI
            0xEDA1 => {
                //self.cpi();

            }

            _ => {}
        }

        match opcode {
            0xED57 | 0xED5F | 0xED47 | 0xED4F | 0xDDF9 | 0xFDF9 |
            0xDDE5 | 0xFDE5 | 0xDDE1 | 0xFDE1 | 0xDDE3 | 0xFDE3 |
            0xEDA0 | 0xEDB0 | 0xEDA8 | 0xEDB8 => self.pc += 2,
            0xDD46 | 0xFD46 | 0xDD4E | 0xFD4E | 0xDD56 | 0xFD56 |
            0xDD5E | 0xFD5E | 0xDD66 | 0xFD66 | 0xDD6E | 0xFD6E |
            0xDD7E | 0xFD7E |
            0xDD70 | 0xDD71 | 0xDD72 | 0xDD73 | 0xDD74 | 0xDD75 |
            0xDD77 |
            0xFD70 | 0xFD71 | 0xFD72 | 0xFD73 | 0xFD74 | 0xFD75 |
            0xFD77 => self.pc += 3,
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

            0x76 => self.halt = true,                                               // HLT

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

            _ => {},
        }

        match opcode {
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => self.pc += 2,
            0x32 | 0x01 | 0x11 | 0x21 | 0x31 | 0x2A | 0x22 => self.pc += 3,
            _ => self.pc +=1,
        }

        cycles
    }
}