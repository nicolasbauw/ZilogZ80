# zilog_z80

[![Current Crates.io Version](https://img.shields.io/crates/v/zilog_z80.svg)](https://crates.io/crates/zilog_z80)
[![Current docs Version](https://docs.rs/zilog_z80/badge.svg)](https://docs.rs/zilog_z80)
[![Downloads badge](https://img.shields.io/crates/d/zilog_z80.svg)](https://crates.io/crates/zilog_z80)

This is a Z80 emulator.

Example for a small loop:
```rust
use zilog_z80::cpu::CPU;
let mut c = CPU::new(0xFFFF);
c.reg.pc = 0x0100;                  // sets pc to 0x0100
// Here we create a small machine code program for demo purpose.
// Usually you will rather load an assembled code in memory with the load_bin function.
c.bus.write_byte(0x0100, 0x3e);     // LD A,0x0F
c.bus.write_byte(0x0101, 0x0F);
c.bus.write_byte(0x0102, 0x3d);     // DEC A
c.bus.write_byte(0x0103, 0xc2);     // JP NZ,0x0102
c.bus.write_word(0x0104, 0x0102);
c.bus.write_byte(0x0106, 0xc9);     // RET
loop {
    c.execute();
    if c.reg.pc == 0x0000 { break }
}
```

For IO and MMIO examples see my [demonstration TRS-80 emulator.](https://github.com/nicolasbauw/TRS-80)

License: MIT
