use std::{error::Error, process};
use zilog_z80::{bus::Bus, cpu::CPU};

fn main() {
    if let Err(e) = load_execute() {
        println!("{}", e);
        process::exit(1);
    }
}

fn load_execute() -> Result<(), Box<dyn Error>> {
    let bus = std::rc::Rc::new(std::cell::RefCell::new(Bus::new(0xFFFF)));
    let mut c = CPU::new(bus);
    c.debug.opcode = true;

    // Loads assembled program into memory
    c.bus.borrow_mut().load_bin("bin/int_im2.bin", 0)?;

    for _ in 0..9 {
        c.execute();
        if c.debug.opcode {
            println!("{}", c.debug.string);
        }
    }
    c.int_request(0x02);

    loop {
        c.execute();
        if c.debug.opcode {
            println!("{}", c.debug.string);
        }
        if c.reg.pc == 0x0000 {
            break;
        }
    }
    Ok(())
}
