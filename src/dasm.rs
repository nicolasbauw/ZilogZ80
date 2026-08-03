//! Désassembleur Z80.
//!
//! Le décodage suit la structure du jeu d'instructions plutôt qu'une table de
//! libellés par préfixe. Chaque opcode se découpe en champs de bits qui
//! désignent directement le registre, la condition ou l'opération concernée :
//!
//! ```text
//!   7 6 5 4 3 2 1 0
//!   x x y y y z z z
//!       p p q
//! ```
//!
//! C'est ainsi que le composant lui-même le décode, et cela couvre d'un seul
//! tenant les préfixes DD/FD/CB/ED ainsi que les instructions non documentées
//! (SLL, moitiés de IX/IY, doubles opérations de DD CB), qu'une table plate
//! oblige à énumérer une par une — donc à en oublier.
//!
//! Le libellé est précédé des octets de l'instruction, sur une colonne de
//! largeur fixe, pour que le désassemblage d'une plage reste aligné.

use crate::bus::Bus;

/// Largeur de la colonne des octets, avant le libellé.
const BYTES_COLUMN: usize = 14;

/// Registres 8 bits désignés par les champs y et z. L'entrée 6 est l'accès
/// mémoire, que le préfixe DD/FD remplace par (IX+d) ou (IY+d).
const R: [&str; 8] = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];

/// Paires 16 bits désignées par le champ p, dans les deux tables qu'utilise
/// le Z80 : celle où 3 vaut SP, et celle où 3 vaut AF (PUSH/POP).
const RP: [&str; 4] = ["BC", "DE", "HL", "SP"];
const RP2: [&str; 4] = ["BC", "DE", "HL", "AF"];

/// Conditions désignées par le champ y.
const CC: [&str; 8] = ["NZ", "Z", "NC", "C", "PO", "PE", "P", "M"];

/// Opérations de l'unité arithmétique, avec leur destination implicite.
const ALU: [&str; 8] = [
    "ADD A,", "ADC A,", "SUB A,", "SBC A,", "AND ", "XOR ", "OR ", "CP ",
];

/// Décalages et rotations du préfixe CB. SLL n'est pas documentée mais existe
/// bel et bien dans le silicium, et les jeux s'en servent.
const ROT: [&str; 8] = ["RLC", "RRC", "RL", "RR", "SLA", "SRA", "SLL", "SRL"];

/// Mode d'interruption désigné par le champ y de ED xx x110.
const IM: [&str; 8] = ["0", "0", "1", "2", "0", "0", "1", "2"];

/// Instructions de transfert par blocs : [y-4][z].
const BLI: [[&str; 4]; 4] = [
    ["LDI", "CPI", "INI", "OUTI"],
    ["LDD", "CPD", "IND", "OUTD"],
    ["LDIR", "CPIR", "INIR", "OTIR"],
    ["LDDR", "CPDR", "INDR", "OTDR"],
];

/// Registre d'index en vigueur, imposé par un préfixe DD ou FD.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Index {
    None,
    Ix,
    Iy,
}

impl Index {
    /// Nom de la paire 16 bits : HL, ou le registre d'index qui la remplace.
    fn pair(self) -> &'static str {
        match self {
            Index::None => "HL",
            Index::Ix => "IX",
            Index::Iy => "IY",
        }
    }

    fn prefixed(self) -> bool {
        self != Index::None
    }
}

/// Lecteur d'octets qui retient combien il en a consommé : la longueur de
/// l'instruction est le sous-produit naturel de son décodage.
struct Fetch<'a, B: Bus + ?Sized> {
    bus: &'a B,
    address: u16,
    len: u8,
}

impl<B: Bus + ?Sized> Fetch<'_, B> {
    fn byte(&mut self) -> u8 {
        let b = self.bus.read_byte(self.address.wrapping_add(self.len as u16));
        self.len += 1;
        b
    }

    fn word(&mut self) -> u16 {
        let lo = self.byte();
        let hi = self.byte();
        u16::from_le_bytes([lo, hi])
    }

    /// Adresse visée par un saut relatif, calculée depuis la fin de
    /// l'instruction comme le fait le processeur.
    fn relative(&mut self) -> u16 {
        let d = self.byte() as i8;
        self.address
            .wrapping_add(self.len as u16)
            .wrapping_add(d as u16)
    }
}

/// Déplacement signé d'un accès indexé, tel qu'on l'écrit en assembleur.
fn displacement(d: u8) -> String {
    let d = d as i8;
    if d < 0 {
        format!("-${:02X}", d.unsigned_abs())
    } else {
        format!("+${:02X}", d)
    }
}

/// Opérande 8 bits désignée par un champ y ou z.
///
/// Sous préfixe DD/FD, l'accès mémoire devient indexé — et consomme alors le
/// déplacement, d'où le lecteur en paramètre — tandis que H et L désignent les
/// moitiés du registre d'index.
fn operand<B: Bus + ?Sized>(code: u8, index: Index, f: &mut Fetch<B>) -> String {
    match (code, index) {
        (6, Index::None) => "(HL)".to_string(),
        (6, _) => {
            let d = f.byte();
            format!("({}{})", index.pair(), displacement(d))
        }
        (4, Index::Ix) => "IXh".to_string(),
        (5, Index::Ix) => "IXl".to_string(),
        (4, Index::Iy) => "IYh".to_string(),
        (5, Index::Iy) => "IYl".to_string(),
        _ => R[code as usize].to_string(),
    }
}

/// Paire 16 bits désignée par le champ p, HL cédant la place au registre
/// d'index quand un préfixe est présent.
fn pair(p: u8, index: Index, table: &[&str; 4]) -> String {
    if p == 2 {
        index.pair().to_string()
    } else {
        table[p as usize].to_string()
    }
}

/// Désassemble l'instruction située à `address`.
///
/// Renvoie son libellé, précédé de ses octets en hexadécimal, ainsi que sa
/// longueur en octets — de quoi enchaîner sur l'instruction suivante.
pub fn dasm<B: Bus + ?Sized>(bus: &B, address: u16) -> (String, u8) {
    let mut f = Fetch {
        bus,
        address,
        len: 0,
    };
    let text = decode(&mut f, Index::None);
    let len = f.len;

    let mut bytes = String::new();
    for i in 0..len {
        let b = bus.read_byte(address.wrapping_add(i as u16));
        // Un préfixe et l'octet qu'il qualifie forment un seul groupe : c'est
        // ce qui distingue à l'œil ED B0 de deux instructions d'un octet.
        if i > 0 && !(i == 1 && is_prefix(bus.read_byte(address))) {
            bytes.push(' ');
        }
        bytes.push_str(&format!("{b:02X}"));
    }

    (format!("{bytes:<BYTES_COLUMN$}{text}"), len)
}

fn is_prefix(opcode: u8) -> bool {
    matches!(opcode, 0xCB | 0xDD | 0xED | 0xFD)
}

/// Décode une instruction, éventuellement déjà sous préfixe DD ou FD.
fn decode<B: Bus + ?Sized>(f: &mut Fetch<B>, index: Index) -> String {
    let op = f.byte();
    let (x, y, z) = (op >> 6, (op >> 3) & 7, op & 7);
    let (p, q) = (y >> 1, y & 1);

    match x {
        0 => match z {
            0 => match y {
                0 => "NOP".to_string(),
                1 => "EX AF,AF'".to_string(),
                2 => format!("DJNZ ${:04X}", f.relative()),
                3 => format!("JR ${:04X}", f.relative()),
                _ => format!("JR {},${:04X}", CC[(y - 4) as usize], f.relative()),
            },
            1 => {
                if q == 0 {
                    let rp = pair(p, index, &RP);
                    format!("LD {},${:04X}", rp, f.word())
                } else {
                    format!("ADD {},{}", index.pair(), pair(p, index, &RP))
                }
            }
            2 => match (q, p) {
                (0, 0) => "LD (BC),A".to_string(),
                (0, 1) => "LD (DE),A".to_string(),
                (0, 2) => format!("LD (${:04X}),{}", f.word(), index.pair()),
                (0, _) => format!("LD (${:04X}),A", f.word()),
                (_, 0) => "LD A,(BC)".to_string(),
                (_, 1) => "LD A,(DE)".to_string(),
                (_, 2) => format!("LD {},(${:04X})", index.pair(), f.word()),
                (_, _) => format!("LD A,(${:04X})", f.word()),
            },
            3 => {
                let rp = pair(p, index, &RP);
                if q == 0 {
                    format!("INC {rp}")
                } else {
                    format!("DEC {rp}")
                }
            }
            4 => format!("INC {}", operand(y, index, f)),
            5 => format!("DEC {}", operand(y, index, f)),
            6 => {
                // Le déplacement précède la valeur immédiate : LD (IX+d),n
                // s'écrit DD 36 d n.
                let dst = operand(y, index, f);
                format!("LD {},${:02X}", dst, f.byte())
            }
            _ => ["RLCA", "RRCA", "RLA", "RRA", "DAA", "CPL", "SCF", "CCF"][y as usize].to_string(),
        },
        1 => {
            if y == 6 && z == 6 {
                return "HALT".to_string();
            }
            // Une seule des deux opérandes peut être indexée : dans
            // LD H,(IX+d), le H reste un vrai H.
            let (dst, src) = match (y, z) {
                (6, _) => {
                    let dst = operand(6, index, f);
                    (dst, operand(z, Index::None, f))
                }
                (_, 6) => {
                    let src = operand(6, index, f);
                    (operand(y, Index::None, f), src)
                }
                _ => (operand(y, index, f), operand(z, index, f)),
            };
            format!("LD {dst},{src}")
        }
        2 => format!("{}{}", ALU[y as usize], operand(z, index, f)),
        _ => match z {
            0 => format!("RET {}", CC[y as usize]),
            1 => {
                if q == 0 {
                    format!("POP {}", pair(p, index, &RP2))
                } else {
                    match p {
                        0 => "RET".to_string(),
                        1 => "EXX".to_string(),
                        2 => format!("JP ({})", index.pair()),
                        _ => format!("LD SP,{}", index.pair()),
                    }
                }
            }
            2 => format!("JP {},${:04X}", CC[y as usize], f.word()),
            3 => match y {
                0 => format!("JP ${:04X}", f.word()),
                1 => decode_cb(f, index),
                2 => format!("OUT (${:02X}),A", f.byte()),
                3 => format!("IN A,(${:02X})", f.byte()),
                4 => format!("EX (SP),{}", index.pair()),
                5 => "EX DE,HL".to_string(),
                6 => "DI".to_string(),
                _ => "EI".to_string(),
            },
            4 => format!("CALL {},${:04X}", CC[y as usize], f.word()),
            5 => {
                if q == 0 {
                    format!("PUSH {}", pair(p, index, &RP2))
                } else {
                    match p {
                        0 => format!("CALL ${:04X}", f.word()),
                        1 => decode(f, Index::Ix),
                        2 => decode_ed(f),
                        _ => decode(f, Index::Iy),
                    }
                }
            }
            6 => format!("{}${:02X}", ALU[y as usize], f.byte()),
            _ => format!("RST ${:02X}", y * 8),
        },
    }
}

/// Instructions préfixées CB : rotations, décalages et opérations sur bits.
///
/// Sous DD/FD, le déplacement se lit AVANT l'opcode, et l'opération porte
/// toujours sur la case mémoire indexée : le champ z ne désigne alors plus
/// l'opérande mais un registre où recopier le résultat, forme non documentée
/// mais bien réelle.
fn decode_cb<B: Bus + ?Sized>(f: &mut Fetch<B>, index: Index) -> String {
    let indexed = if index.prefixed() {
        let d = f.byte();
        Some(format!("({}{})", index.pair(), displacement(d)))
    } else {
        None
    };

    let op = f.byte();
    let (x, y, z) = (op >> 6, (op >> 3) & 7, op & 7);
    let target = indexed
        .clone()
        .unwrap_or_else(|| R[z as usize].to_string());

    let text = match x {
        0 => format!("{} {}", ROT[y as usize], target),
        1 => format!("BIT {},{}", y, target),
        2 => format!("RES {},{}", y, target),
        _ => format!("SET {},{}", y, target),
    };

    // BIT n'écrit nulle part : sa forme indexée n'a pas de copie.
    match indexed {
        Some(_) if z != 6 && x != 1 => format!("LD {},{}", R[z as usize], text),
        _ => text,
    }
}

/// Instructions préfixées ED.
///
/// Les trous de la table ne sont pas des instructions : le processeur les
/// traverse sans rien faire, en deux octets.
fn decode_ed<B: Bus + ?Sized>(f: &mut Fetch<B>) -> String {
    let op = f.byte();
    let (x, y, z) = (op >> 6, (op >> 3) & 7, op & 7);
    let (p, q) = (y >> 1, y & 1);

    match x {
        1 => match z {
            0 => {
                if y == 6 {
                    "IN (C)".to_string()
                } else {
                    format!("IN {},(C)", R[y as usize])
                }
            }
            1 => {
                if y == 6 {
                    "OUT (C),0".to_string()
                } else {
                    format!("OUT (C),{}", R[y as usize])
                }
            }
            2 => {
                let mnemonic = if q == 0 { "SBC" } else { "ADC" };
                format!("{} HL,{}", mnemonic, RP[p as usize])
            }
            3 => {
                let nn = f.word();
                if q == 0 {
                    format!("LD (${:04X}),{}", nn, RP[p as usize])
                } else {
                    format!("LD {},(${:04X})", RP[p as usize], nn)
                }
            }
            4 => "NEG".to_string(),
            5 => {
                if y == 1 {
                    "RETI".to_string()
                } else {
                    "RETN".to_string()
                }
            }
            6 => format!("IM {}", IM[y as usize]),
            _ => ["LD I,A", "LD R,A", "LD A,I", "LD A,R", "RRD", "RLD", "NOP", "NOP"][y as usize]
                .to_string(),
        },
        2 if z <= 3 && y >= 4 => BLI[(y - 4) as usize][z as usize].to_string(),
        _ => "NOP".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::FlatBus;

    fn disassemble(bytes: &[u8]) -> (String, u8) {
        let mut bus = FlatBus::new(0xFFFF);
        for (i, b) in bytes.iter().enumerate() {
            bus.write_byte(0x100 + i as u16, *b);
        }
        let (s, len) = dasm(&bus, 0x100);
        (s[BYTES_COLUMN..].to_string(), len)
    }

    fn check(bytes: &[u8], expected: &str) {
        let (got, len) = disassemble(bytes);
        assert_eq!(got, expected, "octets {bytes:02X?}");
        assert_eq!(
            len as usize,
            bytes.len(),
            "longueur de {expected} ({bytes:02X?})"
        );
    }

    #[test]
    fn unprefixed_instructions() {
        check(&[0x00], "NOP");
        check(&[0x08], "EX AF,AF'");
        check(&[0x76], "HALT");
        check(&[0x78], "LD A,B");
        check(&[0x7E], "LD A,(HL)");
        check(&[0x36, 0x2A], "LD (HL),$2A");
        check(&[0x21, 0x34, 0x12], "LD HL,$1234");
        check(&[0x22, 0x00, 0xC0], "LD ($C000),HL");
        check(&[0x2A, 0x00, 0xC0], "LD HL,($C000)");
        check(&[0x32, 0x00, 0xC0], "LD ($C000),A");
        check(&[0x09], "ADD HL,BC");
        check(&[0x34], "INC (HL)");
        check(&[0xC9], "RET");
        check(&[0xC0], "RET NZ");
        check(&[0xC3, 0x00, 0x90], "JP $9000");
        check(&[0xCA, 0x00, 0x90], "JP Z,$9000");
        check(&[0xCD, 0x00, 0x90], "CALL $9000");
        check(&[0xE9], "JP (HL)");
        check(&[0xEB], "EX DE,HL");
        check(&[0xE3], "EX (SP),HL");
        check(&[0xD3, 0x7F], "OUT ($7F),A");
        check(&[0xDB, 0x7F], "IN A,($7F)");
        check(&[0xC5], "PUSH BC");
        check(&[0xF1], "POP AF");
        check(&[0xFF], "RST $38");
        check(&[0xD6, 0x80], "SUB A,$80");
        check(&[0xE6, 0x1F], "AND $1F");
        check(&[0xFE, 0x20], "CP $20");
        check(&[0xB7], "OR A");
        check(&[0x2F], "CPL");
    }

    /// Les sauts relatifs s'expriment par leur cible, pas par leur
    /// déplacement : c'est la seule forme exploitable quand on lit du code.
    #[test]
    fn relative_jumps_show_their_target() {
        check(&[0x10, 0x03], "DJNZ $0105");
        check(&[0x10, 0xFE], "DJNZ $0100");
        check(&[0x18, 0x00], "JR $0102");
        check(&[0x20, 0x7F], "JR NZ,$0181");
        check(&[0x38, 0x80], "JR C,$0082");
    }

    #[test]
    fn cb_prefixed_instructions() {
        check(&[0xCB, 0x00], "RLC B");
        check(&[0xCB, 0x06], "RLC (HL)");
        check(&[0xCB, 0x30], "SLL B"); // non documentée
        check(&[0xCB, 0x48], "BIT 1,B");
        check(&[0xCB, 0x86], "RES 0,(HL)");
        check(&[0xCB, 0xFE], "SET 7,(HL)");
    }

    /// Le cas qui manquait : opérations sur bits d'un octet indexé. Un jeu qui
    /// s'en sert pour ses drapeaux devenait illisible en plein débogage.
    #[test]
    fn indexed_bit_instructions() {
        check(&[0xDD, 0xCB, 0x2D, 0x86], "RES 0,(IX+$2D)");
        check(&[0xDD, 0xCB, 0x2D, 0x8E], "RES 1,(IX+$2D)");
        check(&[0xFD, 0xCB, 0x02, 0xC6], "SET 0,(IY+$02)");
        check(&[0xDD, 0xCB, 0xFE, 0x46], "BIT 0,(IX-$02)");
        check(&[0xDD, 0xCB, 0x00, 0x06], "RLC (IX+$00)");
        // Forme non documentée : le résultat est aussi rangé dans un registre.
        check(&[0xDD, 0xCB, 0x04, 0x00], "LD B,RLC (IX+$04)");
        check(&[0xDD, 0xCB, 0x04, 0x81], "LD C,RES 0,(IX+$04)");
    }

    #[test]
    fn indexed_instructions() {
        check(&[0xDD, 0x21, 0x34, 0x12], "LD IX,$1234");
        check(&[0xFD, 0x21, 0x63, 0x83], "LD IY,$8363");
        check(&[0xDD, 0x7E, 0x08], "LD A,(IX+$08)");
        check(&[0xDD, 0x77, 0x00], "LD (IX+$00),A");
        check(&[0xDD, 0x36, 0x0C, 0x00], "LD (IX+$0C),$00");
        check(&[0xDD, 0x34, 0xFF], "INC (IX-$01)");
        check(&[0xDD, 0x23], "INC IX");
        check(&[0xDD, 0x19], "ADD IX,DE");
        check(&[0xDD, 0xE5], "PUSH IX");
        check(&[0xDD, 0xE9], "JP (IX)");
        check(&[0xDD, 0xB6, 0x2D], "OR (IX+$2D)");
        // Moitiés du registre d'index, non documentées.
        check(&[0xDD, 0x7C], "LD A,IXh");
        check(&[0xFD, 0x2C], "INC IYl");
    }

    /// Dans LD H,(IX+d), le H reste un vrai H : une seule opérande peut être
    /// indexée. Confondre les deux est l'erreur classique du décodage indexé.
    #[test]
    fn only_the_memory_operand_is_indexed() {
        check(&[0xDD, 0x66, 0x04], "LD H,(IX+$04)");
        check(&[0xDD, 0x74, 0x04], "LD (IX+$04),H");
        check(&[0xDD, 0x65], "LD IXh,IXl");
    }

    #[test]
    fn ed_prefixed_instructions() {
        check(&[0xED, 0x40], "IN B,(C)");
        check(&[0xED, 0x49], "OUT (C),C");
        check(&[0xED, 0x70], "IN (C)");
        check(&[0xED, 0x71], "OUT (C),0");
        check(&[0xED, 0x42], "SBC HL,BC");
        check(&[0xED, 0x4A], "ADC HL,BC");
        check(&[0xED, 0x43, 0x00, 0xC0], "LD ($C000),BC");
        check(&[0xED, 0x5B, 0xC7, 0x82], "LD DE,($82C7)");
        check(&[0xED, 0x44], "NEG");
        check(&[0xED, 0x45], "RETN");
        check(&[0xED, 0x4D], "RETI");
        check(&[0xED, 0x56], "IM 1");
        check(&[0xED, 0x5F], "LD A,R");
        check(&[0xED, 0x67], "RRD");
        check(&[0xED, 0xB0], "LDIR");
        check(&[0xED, 0xB3], "OTIR");
        check(&[0xED, 0xA2], "INI");
        // Trou de la table : deux octets traversés sans effet.
        check(&[0xED, 0x00], "NOP");
    }

    /// Aucun octet ne doit rester indéchiffrable, et la longueur annoncée doit
    /// toujours être exploitable : c'est ce qui permet de désassembler une
    /// plage sans se désaligner.
    #[test]
    fn every_opcode_decodes_with_a_usable_length() {
        let mut bus = FlatBus::new(0xFFFF);
        for first in 0u16..=0xFF {
            for second in 0u16..=0xFF {
                bus.write_byte(0x100, first as u8);
                bus.write_byte(0x101, second as u8);
                bus.write_byte(0x102, 0x12);
                bus.write_byte(0x103, 0x34);
                let (text, len) = dasm(&bus, 0x100);
                assert!(
                    (1..=4).contains(&len),
                    "{first:02X} {second:02X} : longueur {len}"
                );
                let mnemonic = &text[BYTES_COLUMN..];
                assert!(
                    !mnemonic.is_empty() && !mnemonic.contains('?'),
                    "{first:02X} {second:02X} : {mnemonic:?}"
                );
            }
        }
    }
}
