use crate::cpu::{CPU, signed_to_abs};
use crate::bit;

impl CPU {
    /// Disassembles code at (address)
    pub fn dasm_1byte(&self, address: u16) -> String {
        let opcode = self.bus.read_byte(address);
        match opcode {
            // 8-Bit Load Group
            // LD r,r'      LD r,(HL)
            0x40 => String::from("40        LD B,B"),           // LD B,B
            0x41 => String::from("41        LD B,C"),           // LD B,C
            0x42 => String::from("42        LD B,D"),           // LD B,D
            0x43 => String::from("43        LD B,E"),           // LD B,E
            0x44 => String::from("44        LD B,H"),           // LD B,H
            0x45 => String::from("45        LD B,L"),           // LD B,L
            0x46 => {                                           // LD B,(HL)
                let addr = self.reg.get_hl();
                format!("46        LD B,(${:04X})", addr)
            },
            0x47 => String::from("47        LD B,A"),           // LD B,A

            0x48 => String::from("48        LD C,B"),           // LD C,B
            0x49 => String::from("49        LD C,C"),           // LD C,C
            0x4A => String::from("4A        LD C,D"),           // LD C,D
            0x4B => String::from("4B        LD C,E"),           // LD C,E
            0x4C => String::from("4C        LD C,H"),           // LD C,H
            0x4D => String::from("4D        LD C,L"),           // LD C,L
            0x4E => {                                           // LD C,(HL)
                let addr = self.reg.get_hl();
                format!("4E        LD C,(${:04X})", addr)
            }
            0x4F => String::from("4F        LD C,A"),            // LD C,A

            0x50 => String::from("50        LD D,B"),            // LD D,B
            0x51 => String::from("51        LD D,C"),            // LD D,C
            0x52 => String::from("52        LD D,D"),            // LD D,D
            0x53 => String::from("53        LD D,E"),            // LD D,E
            0x54 => String::from("54        LD D,H"),            // LD D,H
            0x55 => String::from("55        LD D,L"),            // LD D,L
            0x56 => {                                            // LD D,(HL)
                let addr = self.reg.get_hl();
                format!("4E        LD D,(${:04X})", addr)
            }
            0x57 => String::from("57        LD D,A"),            // LD D,A

            0x58 => String::from("58        LD E,B"),            // LD E,B
            0x59 => String::from("59        LD E,C"),            // LD E,C
            0x5A => String::from("5A        LD E,D"),            // LD E,D
            0x5B => String::from("5B        LD E,E"),            // LD E,E
            0x5C => String::from("5C        LD E,H"),            // LD E,H
            0x5D => String::from("5D        LD E,L"),            // LD E,L
            0x5E => {                                            // LD E,(HL)
                let addr = self.reg.get_hl();
                format!("5E        LD E,(${:04X})", addr)
            }
            0x5F => String::from("5F        LD E,A"),            // LD E,A

            0x60 => String::from("60        LD H,B"),            // LD H,B
            0x61 => String::from("61        LD H,C"),            // LD H,C
            0x62 => String::from("62        LD H,D"),            // LD H,D
            0x63 => String::from("63        LD H,E"),            // LD H,E
            0x64 => String::from("64        LD H,H"),            // LD H,H
            0x65 => String::from("65        LD H,L"),            // LD H,L
            0x66 => {                                            // LD H,(HL)
                let addr = self.reg.get_hl();
                format!("66        LD H,(${:04X})", addr)
            }
            0x67 => String::from("67        LD H,A"),            // LD H,A

            0x68 => String::from("68        LD L,B"),            // LD L,B
            0x69 => String::from("69        LD L,C"),            // LD L,C
            0x6A => String::from("6A        LD L,D"),            // LD L,D
            0x6B => String::from("6B        LD L,E"),            // LD L,E
            0x6C => String::from("6C        LD L,H"),            // LD L,H
            0x6D => String::from("6D        LD L,L"),            // LD L,L
            0x6E => {                                            // LD L,(HL)
                let addr = self.reg.get_hl();
                format!("6E        LD L,(${:04X})", addr)
            }
            0x6F => String::from("6F        LD L,A"),            // LD L,A

            0x78 => String::from("78        LD A,B"),            // LD A,B
            0x79 => String::from("79        LD A,C"),            // LD A,C
            0x7A => String::from("7A        LD A,D"),            // LD A,D
            0x7B => String::from("7B        LD A,E"),            // LD A,E
            0x7C => String::from("7C        LD A,H"),            // LD A,H
            0x7D => String::from("7D        LD A,L"),            // LD A,L
            0x7E => {                                            // LD A,(HL)
                let addr = self.reg.get_hl();
                format!("7E        LD A,(${:04X})", addr)
            }
            0x7F => String::from("7F        LD A,A"),            // LD A,A

            // LD (HL),r
            0x70 => {                                            // LD (HL), B
                let addr = self.reg.get_hl();
                format!("70        LD (${:04X}),B", addr)
            },
            0x71 => {                                            // LD (HL), C
                let addr = self.reg.get_hl();
                format!("71        LD (${:04X}),C", addr)
            },
            0x72 => {                                            // LD (HL), D
                let addr = self.reg.get_hl();
                format!("72        LD (${:04X}),D", addr)
            },
            0x73 => {                                            // LD (HL), E
                let addr = self.reg.get_hl();
                format!("73        LD (${:04X}),E", addr)
            },
            0x74 => {                                            // LD (HL), H
                let addr = self.reg.get_hl();
                format!("74        LD (${:04X}),H", addr)
            },
            0x75 => {                                            // LD (HL), L
                let addr = self.reg.get_hl();
                format!("75        LD (${:04X}),L", addr)
            },

            0x77 => {                                            // LD (HL), A
                let addr = self.reg.get_hl();
                format!("77        LD (${:04X}),A", addr)
            },

            // LD r,n
            0x06 => {                                                               // LD B,n
                let data = self.bus.read_byte(address + 1);
                format!("06        LD B,${:02X}", data)
            },
            0x0E => {                                                               // LD C,n
                let data = self.bus.read_byte(address + 1);
                format!("0E        LD C,${:02X}", data)
            },
            0x16 => {                                                               // LD D,n
                let data = self.bus.read_byte(address + 1);
                format!("16        LD D,${:02X}", data)
            },
            0x1E => {                                                               // LD E,n
                let data = self.bus.read_byte(address + 1);
                format!("1E        LD E,${:02X}", data)
            },
            0x26 => {                                                               // LD H,n
                let data = self.bus.read_byte(address + 1);
                format!("26        LD H,${:02X}", data)
            },
            0x2E => {                                                               // LD L,n
                let data = self.bus.read_byte(address + 1);
                format!("2E        LD L,${:02X}", data)
            },
            0x36 => {                                                               // LD (HL),n
                let data = self.bus.read_byte(address + 1);
                let addr = self.reg.get_hl();
                format!("36        LD (${:04X}),{:02X}", addr, data)
            },
            0x3E => {                                                               // LD A,n
                let data = self.bus.read_byte(address + 1);
                format!("3E        LD A,${:02X}", data)
            },

            // LD A,(BC)
            0x0A => {
                let addr = self.reg.get_bc();
                format!("0A        LD A,(${:04X})", addr)
            },

            // LD A,(DE)
            0x1A => {
                let addr = self.reg.get_de();
                format!("1A        LD A,(${:04X})", addr)
            },

            // LD A,(nn)
            0x3A => {
                let addr = self.bus.read_word(address + 1);
                format!("3A        LD A,(${:04X})", addr)
            },

            // LD (BC),A
            0x02 => {
                let addr = self.reg.get_bc();
                format!("02        LD (${:04X}),A", addr)
            },

            // LD (DE),A
            0x12 => {
                let addr = self.reg.get_de();
                format!("12        LD (${:04X}),A", addr)
            },

            // LD (nn),A
            0x32 => {
                let addr = self.bus.read_word(address + 1);
                format!("32        LD (${:04X}),A", addr)
            },

            // 16-Bit Load Group
            // LD dd,nn
            0x01 => {                                                               // LD BC,nn
                let d16 = self.bus.read_word(address + 1); 
                format!("01        LD BC,${:04X}", d16)
            },
            0x11 => {                                                               // LD DE,nn
                let d16 = self.bus.read_word(address + 1); 
                format!("11        LD DE,${:04X}", d16)
            },
            0x21 => {                                                               // LD HL,nn
                let d16 = self.bus.read_word(address + 1); 
                format!("21        LD HL,${:04X}", d16)
            },
            0x31 => {                                                               // LD SP,nn
                let d16 = self.bus.read_word(address + 1); 
                format!("31        LD SP,${:04X}", d16)
            },

            // LD HL,(nn)
            0x2A => {
                let addr = self.bus.read_word(address + 1);
                format!("2A        LD HL,(${:04X})", addr)
            },

            // LD (nn),HL
            0x22 => {
                let addr = self.bus.read_word(address + 1);
                format!("22        LD (${:04X}),HL", addr)
            },

            // LD SP,HL
            0xF9 => String::from("F9        LD SP,HL"),

            // PUSH qq
            0xC5 => String::from("C5        PUSH BC"),                              // PUSH BC
            0xD5 => String::from("D5        PUSH DE"),                              // PUSH DE
            0xE5 => String::from("E5        PUSH HL"),                              // PUSH HL
            0xF5 => String::from("F5        PUSH AF"),                              // PUSH AF

            // POP qq
            0xC1 => String::from("C1        POP BC"),                               // POP BC
            0xD1 => String::from("D1        POP DE"),                               // POP DE
            0xE1 => String::from("E1        POP HL"),                               // POP HL
            0xF1 => String::from("F1        POP AF"),                               // POP AF

            // Exchange, Block Transfer, and Search Group
            // EX DE,HL
            0xEB => String::from("EB        EX DE,HL"),

            // EX AF,AF'
            0x08 => String::from("08        EX AF,AF'"),

            // EXX
            0xD9 => String::from("D9        EXX"),

            // EX (SP),HL
            0xE3 => String::from("E3        EX (SP),HL"),

            // 8-Bit Arithmetic Group
            // ADD A,r
            0x80 => String::from("80        ADD A,B"),                      // ADD A,B
            0x81 => String::from("81        ADD A,C"),                      // ADD A,C
            0x82 => String::from("82        ADD A,D"),                      // ADD A,D
            0x83 => String::from("83        ADD A,E"),                      // ADD A,E
            0x84 => String::from("84        ADD A,H"),                      // ADD A,H
            0x85 => String::from("85        ADD A,L"),                      // ADD A,L
            0x86 => {                                                       // ADD A,(HL)
                let addr = self.reg.get_hl();
                format!("86        ADD A,(${:04X})", addr)
            },
            0x87 => String::from("87        ADD A,A"),                      // ADD A,A

            // ADD A,n
            0xC6 => {
                let n = self.bus.read_byte(address + 1);
                format!("C6        ADD A,${:02X}", n)
            },

            // ADC A,r
            0x88 => String::from("88        ADC A,B"),                       // ADC A,B
            0x89 => String::from("89        ADC A,C"),                       // ADC A,C
            0x8A => String::from("8A        ADC A,D"),                       // ADC A,D
            0x8B => String::from("8B        ADC A,E"),                       // ADC A,E
            0x8C => String::from("8C        ADC A,H"),                       // ADC A,H
            0x8D => String::from("8D        ADC A,L"),                       // ADC A,L
            0x8E => {                                                        // ADC A,(HL)
                let addr = self.reg.get_hl();
                format!("8E        ADC A,(${:04X})", addr)
            },
            0x8F => String::from("8F        ADC A,A"),                       // ADC A,A

            // ADC a,n
            0xCE => {                                                        // ADC A,(HL)
                let n = self.bus.read_byte(address + 1);
                format!("CE        ADC A,${:02X}", n)
            },

            // SUB s
            0x90 => String::from("90        SUB A,B"),                       // SUB A,B
            0x91 => String::from("91        SUB A,C"),                       // SUB A,C
            0x92 => String::from("92        SUB A,D"),                       // SUB A,D
            0x93 => String::from("93        SUB A,E"),                       // SUB A,E
            0x94 => String::from("94        SUB A,H"),                       // SUB A,H
            0x95 => String::from("95        SUB A,L"),                       // SUB A,L
            0x96 => {                                                        // SUB A,(HL)
                let addr = self.reg.get_hl();
                format!("96        SUB A,(${:04X})", addr)
            },
            0x97 => String::from("97        SUB A,A"),                       // SUB A,A

            0xD6 => {                                                        // SUB A,n
                let n = self.bus.read_byte(address + 1);
                format!("D6        SUB A,${:02X}", n)
            },

            // SBC A,s
            0x98 => String::from("98        SBC A,B"),                       // SBC A,B
            0x99 => String::from("99        SBC A,C"),                       // SBC A,C
            0x9A => String::from("9A        SBC A,D"),                       // SBC A,D
            0x9B => String::from("9B        SBC A,E"),                       // SBC A,E
            0x9C => String::from("9C        SBC A,H"),                       // SBC A,H
            0x9D => String::from("9D        SBC A,L"),                       // SBC A,L
            0x9E => {                                                        // SBC A,(HL)
                let addr = self.reg.get_hl();
                format!("9E        SBC A,(${:04X})", addr)
            },
            0x9F => String::from("9F        SBC A,A"),                       // SBC A,A

            0xDE => {                                                        // SBC A,n
                let n = self.bus.read_byte(address + 1);
                format!("DE        SBC A,${:02X}", n)
            },

            // AND s
            0xA0 => String::from("A0        AND B"),                       // AND B
            0xA1 => String::from("A1        AND C"),                       // AND C
            0xA2 => String::from("A2        AND D"),                       // AND D
            0xA3 => String::from("A3        AND E"),                       // AND E
            0xA4 => String::from("A4        AND H"),                       // AND H
            0xA5 => String::from("A5        AND L"),                       // AND L
            0xA6 => {                                                      // AND (HL)
                let addr = self.reg.get_hl();
                format!("A6        AND (${:04X})", addr)
            },
            0xA7 => String::from("A7        AND L"),                       // AND A

            0xE6 => {                                                      // AND n
                let n = self.bus.read_byte(address + 1);
                format!("E6        AND ${:02X}", n)
            },

            // OR s
            0xB0 => String::from("B0        OR B"),                       // OR B
            0xB1 => String::from("B1        OR C"),                       // OR C
            0xB2 => String::from("B2        OR D"),                       // OR D
            0xB3 => String::from("B3        OR E"),                       // OR E
            0xB4 => String::from("B4        OR H"),                       // OR H
            0xB5 => String::from("B5        OR L"),                       // OR L
            0xB6 => {                                                     // OR (HL)
                let addr = self.reg.get_hl();
                format!("B6        OR (${:04X})", addr)
            },
            0xB7 => String::from("B7        OR A"),                       // OR A

            0xF6 => {                                                     // OR n
                let n = self.bus.read_byte(address + 1);
                format!("F6        OR ${:02X}", n)
            },

            // XOR s
            0xA8 => String::from("A8        XOR B"),                       // XOR B
            0xA9 => String::from("A9        XOR C"),                       // XOR C
            0xAA => String::from("AA        XOR D"),                       // XOR D
            0xAB => String::from("AB        XOR E"),                       // XOR E
            0xAC => String::from("AC        XOR H"),                       // XOR H
            0xAD => String::from("AD        XOR L"),                       // XOR L
            0xAE => {                                                      // XOR (HL)
                let addr = self.reg.get_hl();
                format!("AE        XOR (${:04X})", addr)
            },
            0xAF => String::from("AF        XOR A"),                       // XOR A

            0xEE => {                                                      // XOR n
                let n = self.bus.read_byte(address + 1);
                format!("EE        XOR ${:02X}", n)
            },

            // CMP s
            0xB8 => String::from("B8        CP B"),                       // CP B
            0xB9 => String::from("B9        CP C"),                       // CP C
            0xBA => String::from("BA        CP D"),                       // CP D
            0xBB => String::from("BB        CP E"),                       // CP E
            0xBC => String::from("BC        CP H"),                       // CP H
            0xBD => String::from("BD        CP L"),                       // CP L
            0xBE => {                                                     // CP (HL)
                let addr = self.reg.get_hl();
                format!("BE        CP (${:04X})", addr)
            },
            0xBF => String::from("BF        CP A"),                       // CP A

            0xFE => {                                                     // CP n
                let n = self.bus.read_byte(address + 1);
                format!("FE        CP ${:02X}", n)
            },

            // INC r
            0x04 => String::from("04        INC B"),                      // INC B
            0x0C => String::from("0C        INC C"),                      // INC C
            0x14 => String::from("14        INC D"),                      // INC D
            0x1C => String::from("1C        INC E"),                      // INC E
            0x24 => String::from("24        INC H"),                      // INC H
            0x2C => String::from("2C        INC L"),                      // INC L
            0x34 => {                                                     // INC (HL)
                let addr = self.reg.get_hl();
                format!("34        INC (${:04X})", addr)
            },
            0x3C => String::from("3C        INC A"),                      // INC A

            // DEC m
            0x05 => String::from("05        DEC B"),                      // DEC B
            0x0D => String::from("0D        DEC C"),                      // DEC C
            0x15 => String::from("15        DEC D"),                      // DEC D
            0x1D => String::from("1D        DEC E"),                      // DEC E
            0x25 => String::from("25        DEC H"),                      // DEC H
            0x2D => String::from("2D        DEC L"),                      // DEC L
            0x35 => {                                                     // DEC (HL)
                let addr = self.reg.get_hl();
                format!("35        DEC (${:04X})", addr)
            },
            0x3D => String::from("3D        DEC A"),                      // DEC A

            // General-Purpose Arithmetic and CPU Control Groups
            // DAA
            0x27 => String::from("27        DAA"),

            // CPL
            0x2F => String::from("2F        CPL"),

            // CCF
            0x3F => String::from("3F        CCF"),

            // SCF
            0x37 => String::from("37        SCF"),

            // NOP
            0x00 => String::from("00        NOP"),

            // HALT
            0x76 => String::from("76        HALT"),

            // DI
            0xF3 => String::from("F3        DI"),

            // EI
            0xFB => String::from("FB        EI"),

            // 16-Bit Arithmetic Group
            // ADD HL,ss
            0x09 => String::from("09        ADD HL,BC"),                    // ADD HL,BC
            0x19 => String::from("19        ADD HL,DE"),                    // ADD HL,DE
            0x29 => String::from("29        ADD HL,HL"),                    // ADD HL,HL
            0x39 => String::from("39        ADD HL,SP"),                    // ADD HL,SP

            // INC ss
            0x03 => String::from("03        INC BC"),                       // INC BC
            0x13 => String::from("13        INC DE"),                       // INC DE
            0x23 => String::from("23        INC HL"),                       // INC HL
            0x33 => String::from("33        INC SP"),                       // INC SP

            // DEC ss
            0x0B => String::from("0B        DEC BC"),                       // DEC BC
            0x1B => String::from("1B        DEC DE"),                       // DEC DE
            0x2B => String::from("2B        DEC HL"),                       // DEC HL
            0x3B => String::from("3B        DEC SP"),                       // DEC SP

            // Rotate and Shift Group
            // RLCA
            0x07 => String::from("07        RLCA"),                       

            // RLA
            0x17 => String::from("17        RLA"),                       

            // RRCA
            0x0F => String::from("0F        RRCA"),                       

            // RRA
            0x1F => String::from("1F        RRA"),                       

            // Jump group
            // JP nn
            0xC3 => {
                let addr = self.bus.read_word(address + 1);
                format!("C3        JP ${:04X}", addr)
            },

            // JP C,nn
            0xDA => {
                let addr = self.bus.read_word(address + 1);
                format!("DA        JP C,${:04X}", addr)
            },

            // JP NC,nn
            0xD2 => {
                let addr = self.bus.read_word(address + 1);
                format!("D2        JP NC,${:04X}", addr)
            },

            // JP Z,nn
            0xCA => {
                let addr = self.bus.read_word(address + 1);
                format!("CA        JP Z,${:04X}", addr)
            },

            // JP NZ,nn
            0xC2 => {
                let addr = self.bus.read_word(address + 1);
                format!("C2        JP NZ,${:04X}", addr)
            },

            // JP M,nn
            0xFA => {
                let addr = self.bus.read_word(address + 1);
                format!("FA        JP M,${:04X}", addr)
            },

            // JP P,nn
            0xF2 => {
                let addr = self.bus.read_word(address + 1);
                format!("F2        JP P,${:04X}", addr)
            },

            // JP PE,nn
            0xEA => {
                let addr = self.bus.read_word(address + 1);
                format!("EA        JP PE,${:04X}", addr)
            },

            // JP PO,nn
            0xE2 => {
                let addr = self.bus.read_word(address + 1);
                format!("E2        JP PO,${:04X}", addr)
            },

            // JR e
            0x18 => {
                let displacement= self.bus.read_byte(address + 1);
                let addr = match bit::get(displacement, 7) {
                    true => address - ( signed_to_abs(displacement) as u16 ),
                    false => address + ( displacement as u16 )
                };
                format!("18        JR ${:04X}", addr)
            },

            // JR C,e
            0x38 => {
                let displacement= self.bus.read_byte(address + 1);
                let addr = match bit::get(displacement, 7) {
                    true => address - ( signed_to_abs(displacement) as u16 ),
                    false => address + ( displacement as u16 )
                };
                format!("38        JR C,${:04X}", addr)
            },

            // JR NC,e
            0x30 => {
                let displacement= self.bus.read_byte(address + 1);
                let addr = match bit::get(displacement, 7) {
                    true => address - ( signed_to_abs(displacement) as u16 ),
                    false => address + ( displacement as u16 )
                };
                format!("30        JR NC,${:04X}", addr)
            },

            // JR Z,e
            0x28 => {
                let displacement= self.bus.read_byte(address + 1);
                let addr = match bit::get(displacement, 7) {
                    true => address - ( signed_to_abs(displacement) as u16 ),
                    false => address + ( displacement as u16 )
                };
                format!("28        JR Z,${:04X}", addr)
            },

            // JR NZ,e
            0x20 => {
                let displacement= self.bus.read_byte(address + 1);
                let addr = match bit::get(displacement, 7) {
                    true => address - ( signed_to_abs(displacement) as u16 ),
                    false => address + ( displacement as u16 )
                };
                format!("20        JR NZ,${:04X}", addr)
            },

            // JP (HL)
            0xE9 => {
                let addr = self.reg.get_hl();
                format!("E9        JP ${:04X}", addr)
            },

            // DJNZ, e
            0x10 => {
                let displacement= self.bus.read_byte(address + 1);
                let addr = match bit::get(displacement, 7) {
                    true => address - ( signed_to_abs(displacement) as u16 ),
                    false => address + ( displacement as u16 )
                };
                format!("10        DJNZ ${:04X}", addr)
            }

            // Call and Return Group
            // CALL nn
            0xCD => {
                let addr = self.bus.read_word(address + 1);
                format!("CD        CALL ${:04X}", addr)
            },

            // CALL C,nn
            0xDC => {
                let addr = self.bus.read_word(address + 1);
                format!("DC        CALL C,${:04X}", addr)
            },

            // CALL NC,nn
            0xD4 => {
                let addr = self.bus.read_word(address + 1);
                format!("D4        CALL NC,${:04X}", addr)
            },

            // CALL Z,nn
            0xCC => {
                let addr = self.bus.read_word(address + 1);
                format!("CC        CALL Z,${:04X}", addr)
            },

            // CALL NZ,nn
            0xC4 => {
                let addr = self.bus.read_word(address + 1);
                format!("C4        CALL NZ,${:04X}", addr)
            },

            // CALL M,nn
            0xFC => {
                let addr = self.bus.read_word(address + 1);
                format!("FC        CALL M,${:04X}", addr)
            },

            // CALL P,nn
            0xF4 => {
                let addr = self.bus.read_word(address + 1);
                format!("F4        CALL P,${:04X}", addr)
            },

            // CALL PE,nn
            0xEC => {
                let addr = self.bus.read_word(address + 1);
                format!("EC        CALL PE,${:04X}", addr)
            },

            // CALL PO,nn
            0xE4 => {
                let addr = self.bus.read_word(address + 1);
                format!("E4        CALL PO,${:04X}", addr)
            },

            // RET
            0xC9 => String::from("C9        RET"),

            // RET C
            0xD8 => String::from("D8        RET C"),

            // RET NC
            0xD0 => String::from("D0        RET NC"),

            // RET Z
            0xC8 => String::from("C8        RET Z"),

            // RET NZ
            0xC0 => String::from("C0        RET NZ"),

            // RET M
            0xF8 => String::from("F8        RET M"),

            // RET P
            0xF0 => String::from("F0        RET P"),

            // RET PE
            0xE8 => String::from("E8        RET PE"),

            // RET PO
            0xE0 => String::from("E0        RET PO"),

            // RST 0
            0xC7 => String::from("C7        RST 0"),

            // RST 08
            0xCF => String::from("CF        RST 08"),

            // RST 10
            0xD7 => String::from("D7        RST 10"),

            // RST 18
            0xDF => String::from("DF        RST 18"),

            // RST 20
            0xE7 => String::from("E7        RST 20"),

            // RST 28
            0xEF => String::from("EF        RST 28"),

            // RST 30
            0xF7 => String::from("F7        RST 30"),

            // RST 38
            0xFF => String::from("FF        RST 38"),

            // Input and Output Group
            // IN A,(n)
            0xDB => {
                let port = self.bus.read_byte(address + 1);
                format!("DB        IN A,(${:02X})", port)
            },

            // OUT (n),A
            0xD3 => {
                let port = self.bus.read_byte(address + 1);
                format!("D3        OUT A,(${:02X})", port)
            },

            _ => String::new()
        }
    }
}