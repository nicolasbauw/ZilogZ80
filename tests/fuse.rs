//! Suite de tests Z80 de l'émulateur FUSE (tests/fuse/tests.in et
//! tests.expected, voir tests/fuse/README pour le format), reprise telle
//! quelle de fuse/z80/tests sur sourceforge.net/p/fuse-emulator. Ces données
//! sont sous GPL-2 (© Philip Kendall et contributeurs) : elles restent dans
//! le dépôt et sont exclues du paquet publié (voir `exclude` dans Cargo.toml).
//!
//! Chaque test part d'un état complet (registres, MEMPTR, I, R, IFF, IM)
//! et d'une mémoire de 64 Ko garnie du motif DE AD BE EF, exécute des
//! instructions jusqu'à atteindre le nombre de T-states demandé (la
//! dernière va à son terme), puis compare l'état final, le nombre de
//! T-states consommés, l'ensemble des octets de mémoire modifiés et les
//! écritures sur les ports.
//!
//! Les événements de contention (MC/PC) sont propres au ZX Spectrum et ne
//! sont pas vérifiés ; les lectures/écritures mémoire individuelles non plus,
//! le trait `Bus` ne datant pas les accès. Seules les écritures de port
//! (PW) sont comparées, dans l'ordre.

use std::cell::RefCell;
use zilog_z80::{bus::Bus, cpu::CPU};

const TESTS_IN: &str = include_str!("fuse/tests.in");
const TESTS_EXPECTED: &str = include_str!("fuse/tests.expected");

struct State {
    /// AF BC DE HL AF' BC' DE' HL' IX IY SP PC MEMPTR
    words: [u16; 13],
    i: u8,
    r: u8,
    iff1: bool,
    iff2: bool,
    im: u8,
    halted: bool,
    tstates: u32,
}

struct TestIn {
    name: String,
    state: State,
    memory: Vec<(u16, Vec<u8>)>,
}

struct TestExpected {
    state: State,
    port_writes: Vec<(u16, u8)>,
    memory: Vec<(u16, Vec<u8>)>,
}

fn hex16(s: &str) -> u16 {
    u16::from_str_radix(s, 16).unwrap_or_else(|_| panic!("hexadécimal invalide : {s}"))
}

fn hex8(s: &str) -> u8 {
    u8::from_str_radix(s, 16).unwrap_or_else(|_| panic!("hexadécimal invalide : {s}"))
}

fn parse_state(regs: &str, misc: &str) -> State {
    let w: Vec<u16> = regs.split_whitespace().map(hex16).collect();
    let m: Vec<&str> = misc.split_whitespace().collect();
    assert_eq!(w.len(), 13, "ligne de registres invalide : {regs}");
    assert_eq!(m.len(), 7, "ligne d'état invalide : {misc}");
    State {
        words: w.try_into().unwrap(),
        i: hex8(m[0]),
        r: hex8(m[1]),
        iff1: m[2] != "0",
        iff2: m[3] != "0",
        im: m[4].parse().unwrap(),
        halted: m[5] != "0",
        tstates: m[6].parse().unwrap(),
    }
}

/// Ligne « adresse octet octet ... -1 ».
fn parse_memory_line(line: &str) -> (u16, Vec<u8>) {
    let mut it = line.split_whitespace();
    let addr = hex16(it.next().unwrap());
    let bytes = it.take_while(|t| *t != "-1").map(hex8).collect();
    (addr, bytes)
}

fn parse_in() -> Vec<TestIn> {
    let mut lines = TESTS_IN.lines().filter(|l| !l.trim().is_empty());
    let mut tests = Vec::new();
    while let Some(name) = lines.next() {
        let state = parse_state(lines.next().unwrap(), lines.next().unwrap());
        let mut memory = Vec::new();
        loop {
            let l = lines.next().unwrap();
            if l.trim() == "-1" {
                break;
            }
            memory.push(parse_memory_line(l));
        }
        tests.push(TestIn {
            name: name.trim().to_string(),
            state,
            memory,
        });
    }
    tests
}

/// Les blocs de tests.expected sont séparés par une ligne vide ; les
/// événements sont indentés, les lignes d'état et de mémoire ne le sont pas.
fn parse_expected() -> Vec<(String, TestExpected)> {
    let mut out = Vec::new();
    for block in TESTS_EXPECTED.split("\n\n") {
        let mut lines = block.lines().filter(|l| !l.trim().is_empty());
        let Some(name) = lines.next() else { continue };
        let mut port_writes = Vec::new();
        let mut rest = Vec::new();
        for l in lines {
            if l.starts_with(' ') {
                let t: Vec<&str> = l.split_whitespace().collect();
                if t[1] == "PW" {
                    port_writes.push((hex16(t[2]), hex8(t[3])));
                }
            } else {
                rest.push(l);
            }
        }
        let state = parse_state(rest[0], rest[1]);
        let memory = rest[2..].iter().map(|l| parse_memory_line(l)).collect();
        out.push((
            name.trim().to_string(),
            TestExpected {
                state,
                port_writes,
                memory,
            },
        ));
    }
    out
}

struct TestBus {
    memory: Vec<u8>,
    port_writes: RefCell<Vec<(u16, u8)>>,
}

impl Bus for TestBus {
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data;
    }

    /// Comme le banc de FUSE : le port lu renvoie son octet de poids fort.
    fn read_io(&self, port: u16) -> u8 {
        (port >> 8) as u8
    }

    fn write_io(&mut self, port: u16, data: u8) {
        self.port_writes.borrow_mut().push((port, data));
    }
}

fn initial_memory() -> Vec<u8> {
    [0xDE, 0xAD, 0xBE, 0xEF].repeat(0x4000)
}

fn load_state(c: &mut CPU, s: &State) {
    let w = &s.words;
    c.reg.set_af(w[0]);
    c.reg.set_bc(w[1]);
    c.reg.set_de(w[2]);
    c.reg.set_hl(w[3]);
    c.alt.set_af(w[4]);
    c.alt.set_bc(w[5]);
    c.alt.set_de(w[6]);
    c.alt.set_hl(w[7]);
    c.reg.set_ix(w[8]);
    c.reg.set_iy(w[9]);
    c.reg.sp = w[10];
    c.reg.pc = w[11];
    c.reg.wz = w[12];
    c.reg.i = s.i;
    c.reg.r = s.r;
    c.set_interrupt_state(s.im, s.iff1, s.iff2);
    assert!(!s.halted, "état initial HALT non pris en charge");
}

fn dump_state(c: &CPU, tstates: u32) -> State {
    State {
        words: [
            c.reg.get_af(),
            c.reg.get_bc(),
            c.reg.get_de(),
            c.reg.get_hl(),
            c.alt.get_af(),
            c.alt.get_bc(),
            c.alt.get_de(),
            c.alt.get_hl(),
            c.reg.get_ix(),
            c.reg.get_iy(),
            c.reg.sp,
            c.reg.pc,
            c.reg.wz,
        ],
        i: c.reg.i,
        r: c.reg.r,
        iff1: c.iff1(),
        iff2: c.iff2(),
        im: c.im(),
        halted: c.is_halted(),
        tstates,
    }
}

const WORD_NAMES: [&str; 13] = [
    "AF", "BC", "DE", "HL", "AF'", "BC'", "DE'", "HL'", "IX", "IY", "SP", "PC", "MEMPTR",
];

/// Liste les écarts entre l'état obtenu et l'état attendu.
fn diff_state(got: &State, exp: &State) -> Vec<String> {
    let mut d = Vec::new();
    for (k, name) in WORD_NAMES.iter().enumerate() {
        if got.words[k] != exp.words[k] {
            d.push(format!(
                "{name} {:04x} au lieu de {:04x}",
                got.words[k], exp.words[k]
            ));
        }
    }
    let mut check = |name: &str, g: String, e: String| {
        if g != e {
            d.push(format!("{name} {g} au lieu de {e}"));
        }
    };
    check("I", format!("{:02x}", got.i), format!("{:02x}", exp.i));
    check("R", format!("{:02x}", got.r), format!("{:02x}", exp.r));
    check("IFF1", got.iff1.to_string(), exp.iff1.to_string());
    check("IFF2", got.iff2.to_string(), exp.iff2.to_string());
    check("IM", got.im.to_string(), exp.im.to_string());
    check("HALT", got.halted.to_string(), exp.halted.to_string());
    check("T-states", got.tstates.to_string(), exp.tstates.to_string());
    d
}

fn run_test(t: &TestIn, exp: &TestExpected) -> Vec<String> {
    let mut bus = TestBus {
        memory: initial_memory(),
        port_writes: RefCell::new(Vec::new()),
    };
    for (addr, bytes) in &t.memory {
        for (k, b) in bytes.iter().enumerate() {
            bus.memory[(*addr as usize + k) & 0xFFFF] = *b;
        }
    }
    let before = bus.memory.clone();

    let mut c = CPU::new();
    load_state(&mut c, &t.state);
    let mut tstates = 0;
    while tstates < t.state.tstates {
        tstates += c.execute(&mut bus);
    }

    let mut diffs = diff_state(&dump_state(&c, tstates), &exp.state);

    // Mémoire : l'ensemble exact des octets modifiés.
    let mut expected_mem = before.clone();
    for (addr, bytes) in &exp.memory {
        for (k, b) in bytes.iter().enumerate() {
            expected_mem[(*addr as usize + k) & 0xFFFF] = *b;
        }
    }
    for (a, (got, exp)) in bus.memory.iter().zip(&expected_mem).enumerate() {
        if got != exp {
            diffs.push(format!("mémoire {a:04x} = {got:02x} au lieu de {exp:02x}"));
        }
    }

    let writes = bus.port_writes.into_inner();
    if writes != exp.port_writes {
        diffs.push(format!(
            "écritures de port {writes:04x?} au lieu de {:04x?}",
            exp.port_writes
        ));
    }
    diffs
}

#[test]
fn fuse_test_suite() {
    let tests = parse_in();
    let expected = parse_expected();
    assert_eq!(
        tests.len(),
        expected.len(),
        "tests.in et tests.expected ne comptent pas le même nombre de tests"
    );

    let mut failures = Vec::new();
    for (t, (name, exp)) in tests.iter().zip(&expected) {
        assert_eq!(&t.name, name, "tests.in et tests.expected désynchronisés");
        let diffs = run_test(t, exp);
        if !diffs.is_empty() {
            failures.push(format!("{} : {}", t.name, diffs.join(", ")));
        }
    }

    assert!(
        failures.is_empty(),
        "{} test(s) FUSE en échec sur {} :\n{}",
        failures.len(),
        tests.len(),
        failures.join("\n")
    );
}
