//! This is a Z80 emulator.
//!
//! Example for a small loop:
//! ```rust
//! use zilog_z80::{bus::Bus, cpu::CPU};
//! let bus = std::rc::Rc::new(core::cell::RefCell::new(Bus::new(0xFFFF)));
//! let mut c = CPU::new(bus.clone());
//! c.reg.pc = 0x0100;                  // sets pc to 0x0100
//! // Here we create a small machine code program for demo purpose.
//! // Usually you will rather load an assembled code in memory with the load_bin function.
//! bus.borrow_mut().write_byte(0x0100, 0x3e);     // LD A,0x0F
//! bus.borrow_mut().write_byte(0x0101, 0x0F);
//! bus.borrow_mut().write_byte(0x0102, 0x3d);     // DEC A
//! bus.borrow_mut().write_byte(0x0103, 0xc2);     // JP NZ,0x0102
//! bus.borrow_mut().write_word(0x0104, 0x0102);
//! bus.borrow_mut().write_byte(0x0106, 0xc9);     // RET
//! loop {
//!     c.execute();
//!     if c.reg.pc == 0x0000 { break }
//! }
//! ```
//!
//! Bus has the `Rc<RefCell<Bus>>` type, so it can be shared between all the components of your emulated machine.

mod bit;
pub mod bus;
pub mod cpu;
mod cycles;
mod dasm;
mod flags;
pub mod registers;

#[cfg(test)]
mod test;
