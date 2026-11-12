//! Machine cycles of every instruction, as static tables.
//!
//! [`CPU::execute`](crate::cpu::CPU::execute) runs one whole instruction and
//! only returns its total duration. Some machines need to know what happens
//! inside it: on the Amstrad CPC, for instance, the Gate Array stretches
//! every bus access to a microsecond boundary, so the real duration of an
//! instruction, and the moment of each I/O or memory write, depend on the
//! sequence of its machine cycles, not on their total.
//!
//! [`machine_cycles`] gives that sequence for the instruction that was just
//! executed, from its opcode bytes and the T-states `execute` returned (which
//! tells the variants apart: branch taken or not, repeat or last iteration).
//! Nothing here runs during execution: other users of the crate pay nothing.
//!
//! The tables follow the Zilog user manual (UM0080) and are checked, cycle
//! by cycle, against the bus events of the FUSE test suite (`tests/fuse.rs`).

/// What a machine cycle does on the bus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MCycleKind {
    /// Opcode fetch (M1), prefix bytes included: 4 T-states, the opcode read
    /// in T1-T2, then the memory refresh.
    OpcodeFetch,
    /// Memory read: 3 T-states.
    MemoryRead,
    /// Memory write: 3 T-states.
    MemoryWrite,
    /// I/O read: 4 T-states, the Z80's automatic wait state included.
    IoRead,
    /// I/O write: 4 T-states, the automatic wait state included.
    IoWrite,
    /// Interrupt acknowledge (IM 0, 1 or 2): an M1 cycle lengthened by two
    /// automatic wait states, 6 T-states, during which the interrupting
    /// device supplies the opcode or the vector instead of memory.
    InterruptAcknowledge,
    /// Internal operation, no bus access: any number of T-states.
    Internal,
}

/// One machine cycle: its kind and its length in T-states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MCycle {
    pub kind: MCycleKind,
    pub t_states: u8,
}

const F: MCycle = MCycle {
    kind: MCycleKind::OpcodeFetch,
    t_states: 4,
};
const R: MCycle = MCycle {
    kind: MCycleKind::MemoryRead,
    t_states: 3,
};
const W: MCycle = MCycle {
    kind: MCycleKind::MemoryWrite,
    t_states: 3,
};
const IN: MCycle = MCycle {
    kind: MCycleKind::IoRead,
    t_states: 4,
};
const OUT: MCycle = MCycle {
    kind: MCycleKind::IoWrite,
    t_states: 4,
};
const ACK: MCycle = MCycle {
    kind: MCycleKind::InterruptAcknowledge,
    t_states: 6,
};
const fn internal(t_states: u8) -> MCycle {
    MCycle {
        kind: MCycleKind::Internal,
        t_states,
    }
}
const I1: MCycle = internal(1);
const I2: MCycle = internal(2);
const I4: MCycle = internal(4);
const I5: MCycle = internal(5);
const I7: MCycle = internal(7);

/// HALT, each time it waits: one opcode fetch, repeated.
pub const HALT: &[MCycle] = &[F];
/// Non-maskable interrupt acceptance: an opcode fetch whose byte is ignored,
/// one internal state, then PC pushed (11 T-states).
pub const NMI_ACKNOWLEDGE: &[MCycle] = &[F, I1, W, W];
/// Interrupt acceptance in IM 1 (and IM 0 with an RST opcode): the
/// acknowledge cycle, one internal state, then PC pushed (13 T-states).
pub const IM1_ACKNOWLEDGE: &[MCycle] = &[ACK, I1, W, W];
/// Interrupt acceptance in IM 2: as IM 1, then the handler address read
/// from the vector table (19 T-states).
pub const IM2_ACKNOWLEDGE: &[MCycle] = &[ACK, I1, W, W, R, R];

/// Total length of a sequence of machine cycles, in T-states.
pub fn total_t_states(cycles: &[MCycle]) -> u32 {
    cycles.iter().map(|c| u32::from(c.t_states)).sum()
}

/// The machine cycles of the instruction that `execute` just ran, given its
/// first, second and fourth bytes (the fourth only matters for DD CB / FD CB,
/// where it is the operation) and the T-states `execute` returned. `None` if
/// no variant of that instruction lasts that long (a mismatch between these
/// tables and the CPU).
///
/// Interrupt and NMI acceptance, and HALT, have their own constants
/// ([`IM1_ACKNOWLEDGE`], [`IM2_ACKNOWLEDGE`], [`NMI_ACKNOWLEDGE`], [`HALT`]):
/// the caller knows when one of them happened, these bytes don't.
pub fn machine_cycles(
    first: u8,
    second: u8,
    fourth: u8,
    t_states: u32,
) -> Option<&'static [MCycle]> {
    let matches = |v: &&'static [MCycle]| total_t_states(v) == t_states;
    match first {
        0xCB => Some(cb(second)).filter(matches),
        0xED => ed(second).iter().copied().find(matches),
        0xDD | 0xFD => match second {
            0xCB => Some(ddcb(fourth)).filter(matches),
            // A prefix in front of an instruction that ignores HL: the CPU
            // executes it alone, as a 4-T-state fetch (see `after_prefix`).
            _ => indexed(second)
                .filter(matches)
                .or(Some(&[F][..]).filter(matches)),
        },
        _ => unprefixed(first).iter().copied().find(matches),
    }
}

/// Unprefixed opcodes. Branches list their taken variant first.
fn unprefixed(op: u8) -> &'static [&'static [MCycle]] {
    let (x, y, z) = (op >> 6, (op >> 3) & 7, op & 7);
    let (p, q) = (y >> 1, y & 1);
    match (x, z) {
        (0, 0) => match y {
            0 | 1 => &[&[F]],                     // NOP, EX AF,AF'
            2 => &[&[F, I1, R, I5], &[F, I1, R]], // DJNZ d
            _ => &[&[F, R, I5], &[F, R]],         // JR d, JR cc,d
        },
        (0, 1) if q == 0 => &[&[F, R, R]], // LD rp,nn
        (0, 1) => &[&[F, I7]],             // ADD HL,rp
        (0, 2) => match y {
            0..=3 if q == 0 => &[&[F, W]], // LD (BC),A, LD (DE),A
            0..=3 => &[&[F, R]],           // LD A,(BC), LD A,(DE)
            4 => &[&[F, R, R, W, W]],      // LD (nn),HL
            5 => &[&[F, R, R, R, R]],      // LD HL,(nn)
            6 => &[&[F, R, R, W]],         // LD (nn),A
            _ => &[&[F, R, R, R]],         // LD A,(nn)
        },
        (0, 3) => &[&[F, I2]],                     // INC rp, DEC rp
        (0, 4 | 5) if y == 6 => &[&[F, R, I1, W]], // INC (HL), DEC (HL)
        (0, 4 | 5) => &[&[F]],                     // INC r, DEC r
        (0, 6) if y == 6 => &[&[F, R, W]],         // LD (HL),n
        (0, 6) => &[&[F, R]],                      // LD r,n
        (0, _) => &[&[F]],                         // rotations on A, DAA, CPL, SCF, CCF
        (1, _) if y == 6 && z == 6 => &[&[F]],     // HALT
        (1, 6) => &[&[F, R]],                      // LD r,(HL)
        (1, _) if y == 6 => &[&[F, W]],            // LD (HL),r
        (1, _) => &[&[F]],                         // LD r,r'
        (2, 6) => &[&[F, R]],                      // ALU A,(HL)
        (2, _) => &[&[F]],                         // ALU A,r
        (_, 0) => &[&[F, I1, R, R], &[F, I1]],     // RET cc
        (_, 1) if q == 0 => &[&[F, R, R]],         // POP rp2
        (_, 1) => match p {
            0 => &[&[F, R, R]], // RET
            1 | 2 => &[&[F]],   // EXX, JP (HL)
            _ => &[&[F, I2]],   // LD SP,HL
        },
        (_, 2) => &[&[F, R, R]], // JP cc,nn
        (_, 3) => match y {
            0 => &[&[F, R, R]],               // JP nn
            2 => &[&[F, R, OUT]],             // OUT (n),A
            3 => &[&[F, R, IN]],              // IN A,(n)
            4 => &[&[F, R, R, I1, W, W, I2]], // EX (SP),HL
            _ => &[&[F]],                     // EX DE,HL, DI, EI (CB: prefix)
        },
        (_, 4) => &[&[F, R, R, I1, W, W], &[F, R, R]], // CALL cc,nn
        (_, 5) if q == 0 => &[&[F, I1, W, W]],         // PUSH rp2
        (_, 5) => &[&[F, R, R, I1, W, W]],             // CALL nn (others: prefixes)
        (_, 6) => &[&[F, R]],                          // ALU A,n
        _ => &[&[F, I1, W, W]],                        // RST
    }
}

/// CB-prefixed opcodes.
fn cb(op: u8) -> &'static [MCycle] {
    match (op >> 6, op & 7) {
        (_, z) if z != 6 => &[F, F], // rotations, shifts, BIT, RES, SET on r
        (1, _) => &[F, F, R, I1],    // BIT b,(HL)
        _ => &[F, F, R, I1, W],      // the others on (HL)
    }
}

/// ED-prefixed opcodes. Repeating block instructions list the repeating
/// iteration first, then the last one.
fn ed(op: u8) -> &'static [&'static [MCycle]] {
    let (x, y, z) = (op >> 6, (op >> 3) & 7, op & 7);
    match (x, z) {
        (1, 0) => &[&[F, F, IN]],                       // IN r,(C), IN (C)
        (1, 1) => &[&[F, F, OUT]],                      // OUT (C),r, OUT (C),0
        (1, 2) => &[&[F, F, I7]],                       // SBC HL,rp, ADC HL,rp
        (1, 3) if y & 1 == 0 => &[&[F, F, R, R, W, W]], // LD (nn),rp
        (1, 3) => &[&[F, F, R, R, R, R]],               // LD rp,(nn)
        (1, 5) => &[&[F, F, R, R]],                     // RETN, RETI
        (1, 7) if y < 4 => &[&[F, F, I1]],              // LD I,A, LD R,A, LD A,I, LD A,R
        (1, 7) if y < 6 => &[&[F, F, R, I4, W]],        // RRD, RLD
        (2, 0..=3) if y >= 4 => {
            let repeat = y >= 6;
            match (z, repeat) {
                (0, false) => &[&[F, F, R, W, I2]], // LDI, LDD
                (0, true) => &[&[F, F, R, W, I2, I5], &[F, F, R, W, I2]],
                (1, false) => &[&[F, F, R, I5]], // CPI, CPD
                (1, true) => &[&[F, F, R, I5, I5], &[F, F, R, I5]],
                (2, false) => &[&[F, F, I1, IN, W]], // INI, IND
                (2, true) => &[&[F, F, I1, IN, W, I5], &[F, F, I1, IN, W]],
                (_, false) => &[&[F, F, I1, R, OUT]], // OUTI, OUTD
                (_, true) => &[&[F, F, I1, R, OUT, I5], &[F, F, I1, R, OUT]],
            }
        }
        // NEG, IM n, the undocumented NOPs of the ED table.
        _ => &[&[F, F]],
    }
}

/// DD- or FD-prefixed opcodes that use IX/IY. `None` for the others, which
/// the CPU runs as a lone prefix.
fn indexed(op: u8) -> Option<&'static [MCycle]> {
    let (x, y, z) = (op >> 6, (op >> 3) & 7, op & 7);
    Some(match op {
        0x21 => &[F, F, R, R],                      // LD IX,nn
        0x22 => &[F, F, R, R, W, W],                // LD (nn),IX
        0x2A => &[F, F, R, R, R, R],                // LD IX,(nn)
        0x23 | 0x2B => &[F, F, I2],                 // INC IX, DEC IX
        0x09 | 0x19 | 0x29 | 0x39 => &[F, F, I7],   // ADD IX,rp
        0x24 | 0x25 | 0x2C | 0x2D => &[F, F],       // INC/DEC IXH, IXL
        0x26 | 0x2E => &[F, F, R],                  // LD IXH,n, LD IXL,n
        0x34 | 0x35 => &[F, F, R, I5, R, I1, W],    // INC (IX+d), DEC (IX+d)
        0x36 => &[F, F, R, R, I2, W],               // LD (IX+d),n
        0xE1 => &[F, F, R, R],                      // POP IX
        0xE3 => &[F, F, R, R, I1, W, W, I2],        // EX (SP),IX
        0xE5 => &[F, F, I1, W, W],                  // PUSH IX
        0xE9 => &[F, F],                            // JP (IX)
        0xF9 => &[F, F, I2],                        // LD SP,IX
        0x76 => return None,                        // HALT, not LD (IX+d),(IX+d)
        _ if x == 1 && z == 6 => &[F, F, R, I5, R], // LD r,(IX+d)
        _ if x == 1 && y == 6 => &[F, F, R, I5, W], // LD (IX+d),r
        _ if x == 2 && z == 6 => &[F, F, R, I5, R], // ALU A,(IX+d)
        // LD r,IXH/IXL, LD IXH/IXL,r, ALU A,IXH/IXL; also the prefixed
        // LD r,r' between B, C, D, E and A, which the CPU runs in one go
        // rather than as a lone prefix (same cycles either way).
        _ if x == 1 || (x == 2 && (z == 4 || z == 5)) => &[F, F],
        _ => return None,
    })
}

/// DD CB d op / FD CB d op: prefix and CB are opcode fetches, d and op plain
/// memory reads.
fn ddcb(op: u8) -> &'static [MCycle] {
    if op >> 6 == 1 {
        &[F, F, R, R, I2, R, I1] // BIT b,(IX+d)
    } else {
        &[F, F, R, R, I2, R, I1, W]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::{Bus, FlatBus};
    use crate::cpu::CPU;

    /// Runs one instruction at 0x8000 and returns its T-states, with F = `f`
    /// (so that each condition goes both ways) and B, BC = `count` (so that
    /// DJNZ and the repeating instructions both repeat and stop).
    fn run(bytes: [u8; 4], f: u8, count: u16) -> u32 {
        let mut bus = FlatBus::new(0xFFFF);
        let mut c = CPU::new();
        for (k, b) in bytes.iter().enumerate() {
            bus.write_byte(0x8000 + k as u16, *b);
        }
        c.reg.pc = 0x8000;
        c.reg.sp = 0xC000;
        c.reg.set_af(u16::from(f));
        c.reg.set_bc(count);
        c.reg.set_hl(0x9000);
        c.reg.set_ix(0x9000);
        c.reg.set_iy(0x9000);
        c.execute(&mut bus)
    }

    /// Every opcode of every table, in every variant the CPU produces, has
    /// its machine cycles (the FUSE suite checks their order and timing).
    #[test]
    fn every_instruction_has_its_machine_cycles() {
        let mut missing = Vec::new();
        for first in 0..=255u8 {
            for second in 0..=255u8 {
                // The second byte only matters after a prefix; for DD CB and
                // FD CB, the operation is the fourth byte.
                if !matches!(first, 0xCB | 0xED | 0xDD | 0xFD) && second != 0 {
                    continue;
                }
                let fourths: &[u8] = if matches!(first, 0xDD | 0xFD) && second == 0xCB {
                    &[0x00, 0x46, 0x86, 0xC6, 0x06, 0x7E]
                } else {
                    &[0]
                };
                for &fourth in fourths {
                    for f in [0x00, 0xFF] {
                        for count in [1, 2] {
                            let ticks = run([first, second, 0x05, fourth], f, count);
                            if machine_cycles(first, second, fourth, ticks).is_none() {
                                missing.push(format!(
                                    "{first:02X} {second:02X} ({fourth:02X}): {ticks}"
                                ));
                            }
                        }
                    }
                }
            }
        }
        // Every DD CB / FD CB operation, not just the sample above.
        for prefix in [0xDD, 0xFD] {
            for fourth in 0..=255u8 {
                let ticks = run([prefix, 0xCB, 0x05, fourth], 0, 1);
                if machine_cycles(prefix, 0xCB, fourth, ticks).is_none() {
                    missing.push(format!("{prefix:02X} CB ({fourth:02X}): {ticks}"));
                }
            }
        }
        missing.dedup();
        assert!(
            missing.is_empty(),
            "no machine cycles for:\n{}",
            missing.join("\n")
        );
    }

    /// The acknowledge constants last what `execute` returns for them.
    #[test]
    fn acknowledge_cycles_match_the_cpu() {
        for (im, cycles) in [(1, IM1_ACKNOWLEDGE), (2, IM2_ACKNOWLEDGE)] {
            let mut bus = FlatBus::new(0xFFFF);
            let mut c = CPU::new();
            c.reg.sp = 0xC000;
            c.set_interrupt_state(im, true, true);
            c.int_request(0xFF);
            assert_eq!(c.execute(&mut bus), total_t_states(cycles), "IM {im}");
        }
        let mut bus = FlatBus::new(0xFFFF);
        let mut c = CPU::new();
        c.reg.sp = 0xC000;
        c.nmi_request();
        assert_eq!(c.execute(&mut bus), total_t_states(NMI_ACKNOWLEDGE), "NMI");
        let mut c = CPU::new();
        bus.write_byte(0, 0x76);
        c.execute(&mut bus);
        assert_eq!(c.execute(&mut bus), total_t_states(HALT), "HALT");
    }
}
