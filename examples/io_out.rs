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
    let mut c = CPU::new(bus.clone());
    c.debug.instr_in = true;

    // Loads assembled program into memory
    bus.borrow_mut().load_bin("bin/out_a.bin", 0)?;

    // Demonstration peripheral 0x07 listens data sent by the CPU
    loop {
        c.execute();
        let d = bus.borrow_mut().get_io_out(0x07);
        if d != 0 {
            println!("The 0x07 peripheral received {:#04X} from the CPU", d)
        }
        if c.reg.pc == 0x0000 {
            break;
        }
    }
    Ok(())
}
