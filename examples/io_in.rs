use std::{ error::Error, process };
use zilog_z80::{bus::Bus, cpu::CPU};

fn main() {
    if let Err(e) = load_execute() {
        println!("{}", e);
        process::exit(1);
    }
}

fn load_execute() -> Result<(), Box<dyn Error>> {
    let bus = std::rc::Rc::new(std::cell::RefCell::new(Bus::new(0xFFFF)));
    let mut c = CPU::new(bus.clone());
    c.debug.instr_in = true;

    // Loads assembled program into memory
    bus.borrow_mut().load_bin("bin/in_a.bin", 0)?;

    // A single loop which waits for the 0xDE byte to be sent by the 0x07 peripheral
    loop {
        c.execute();
        bus.borrow_mut().set_io_in(0x07, 0xDE);
        if c.reg.pc == 0x0000 { break }
    }
    Ok(())
}