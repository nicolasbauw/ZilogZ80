use std::{error::Error, process};
use zilog_z80::{bus::Bus, cpu::CPU};

fn main() {
    if let Err(e) = load_execute() {
        println!("{}", e);
        process::exit(1);
    }
}

fn load_execute() -> Result<(), Box<dyn Error>> {
    let mut b = Bus::new(0xFFFF);
    let mut c = CPU::new();
    c.debug.opcode = true;

    // Loads assembled program into memory
    b.load_bin("bin/int_im2.bin", 0)?;

    for _ in 0..9 {
        c.execute(&mut b);
        if c.debug.opcode {
            print!("{}\n", c.debug.string);
        }
    }
    c.int_request(0x02);

    loop {
        c.execute(&mut b);
        if c.debug.opcode {
            print!("{}\n", c.debug.string);
        }
        if c.reg.pc == 0x0000 {
            break;
        }
    }
    Ok(())
}
