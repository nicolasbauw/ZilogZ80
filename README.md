# zilog_z80

[![Current Crates.io Version](https://img.shields.io/crates/v/zilog_z80.svg)](https://crates.io/crates/zilog_z80)
[![Current docs Version](https://docs.rs/zilog_z80/badge.svg)](https://docs.rs/zilog_z80)
[![Downloads badge](https://img.shields.io/crates/d/zilog_z80.svg)](https://crates.io/crates/zilog_z80)

This is a Z80 emulator.

Example for a small loop:
```rust
use zilog_z80::{cpu::CPU, bus::Bus, bus::FlatBus};
let mut b = FlatBus::new(0xFFFF);
let mut c = CPU::new();
c.reg.pc = 0x0100;              // sets pc to 0x0100
// Here we create a small machine code program for demo purpose.
b.write_byte(0x0100, 0x3e);     // LD A,0x0F
b.write_byte(0x0101, 0x0F);
b.write_byte(0x0102, 0x3d);     // DEC A
b.write_byte(0x0103, 0xc2);     // JP NZ,0x0102
b.write_word(0x0104, 0x0102);
b.write_byte(0x0106, 0xc9);     // RET
loop {
    c.execute(&mut b);
    if c.reg.pc == 0x0000 { break }
}
```

FlatBus is used when you just need a very basic bus, and no particular address decoding.  
For more complex address decoding and IO examples see my [Amstrad CPC emulator.](https://github.com/nicolasbauw/amstrad_cpc)

`execute` returns the total duration of an instruction. For machines that
need what happens inside it (on the Amstrad CPC, the Gate Array stretches each
bus access to a microsecond boundary), `machine_cycles` gives the sequence of
machine cycles of the instruction just executed, from its opcode bytes and that
duration: kind (opcode fetch, memory or I/O read or write, interrupt
acknowledge, internal) and length in T-states. They are static tables, checked
against the bus events of the FUSE test suite; `execute` doesn't use them, so
other users pay nothing.

```rust
use zilog_z80::machine_cycles::{machine_cycles, MCycleKind};
// PUSH BC: 11 T-states, a 5-T-state M1 then two memory writes.
let cycles = machine_cycles(0xC5, 0x00, 0x00, 11).unwrap();
assert_eq!(cycles.iter().filter(|c| c.kind == MCycleKind::MemoryWrite).count(), 2);
```

The library provides a disassembler method:

```
cargo run --example disassembler -- bin/inc_dec_ss_ix_iy.bin
   Compiling zilog_z80 v0.18.0 (/home/nicolasb/Dev/ZilogZ80)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running `target/debug/examples/disassembler bin/inc_dec_ss_ix_iy.bin`
01 00 00      LD BC,$0000
11 FF FF      LD DE,$FFFF
21 FF 00      LD HL,$00FF
31 11 11      LD SP,$1111
DD21 FF 0F    LD IX,$0FFF
FD21 34 12    LD IY,$1234
0B            DEC BC
03            INC BC
13            INC DE
1B            DEC DE
23            INC HL
2B            DEC HL
33            INC SP
3B            DEC SP
DD23          INC IX
DD2B          DEC IX
FD23          INC IY
FD2B          DEC IY
```

License: MIT
