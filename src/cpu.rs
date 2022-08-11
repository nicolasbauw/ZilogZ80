use crate::registers::Registers;
use crate::memory::AddressBus;

const CYCLES: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
    7, 7, 7, 7, 7, 7, 4, 7, 4, 4, 4, 4, 4, 4, 7, 4,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub struct CPU {
    pub registers: crate::registers::Registers,
    pub alt_registers: crate::registers::Registers,
    pub i: u8,
    pub r: u8,
    pub ix: u16,
    pub iy: u16,
    pub sp: u16,
    pub pc: u16,
    pub bus: AddressBus,
    pub halt: bool
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
            halt: false
        }
    }

    pub fn execute(&mut self) -> u32 {
        let opcode = self.bus.read_byte(self.pc);
        let cycles = CYCLES[opcode as usize].into();

        match opcode {
            // 8-Bit Load Group
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


            _ => {},
        }

        match opcode {
            _ => self.pc +=1,
        }

        cycles
    }
}