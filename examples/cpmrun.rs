use std::{env, error::Error, process};
use zilog_z80::{bus::Bus, cpu::CPU};

fn main() {
    if let Err(e) = load_execute() {
        println!("{}", e);
        process::exit(1);
    }
}

fn load_execute() -> Result<(), Box<dyn Error>> {
    let a: Vec<String> = env::args().collect();
    let bus = std::rc::Rc::new(std::cell::RefCell::new(Bus::new(0xFFFF)));
    let mut c = CPU::new(bus);
    // Loads assembled program into memory
    c.bus.borrow_mut().load_bin(&a[1], 0x100)?;

    // RET at 0x05 for mocking of CP/M BDOS system calls
    c.bus.borrow_mut().write_word(0x0005, 0xc9);

    // Setting PC to 0x0100 (CP/M Binaries are loaded with a 256 byte offset)
    c.reg.pc = 0x0100;

    /* Setting up stack : by disassembling CP/M software, it seems
    that the $0006 address is read to set the stack by some programs */
    c.bus.borrow_mut().write_word(0x0006, 0xFF00);

    /* Setting up stack in case of the program does not read the $0006 address
    and does not set any stack. */
    c.reg.sp = 0xFF00;

    loop {
        c.execute();
        if c.reg.pc == 0x0005 {
            bdos_call(&c)
        }
        if c.reg.pc == 0x0000 {
            break;
        } //  if CP/M warm boot -> we exit
    }
    Ok(())
}

fn bdos_call(c: &CPU) {
    if c.reg.c == 0x09 {
        let mut a = c.reg.get_de();
        loop {
            let c = c.bus.borrow().read_byte(a);
            if c as char == '$' {
                break;
            } else {
                a += 1;
            }
            print!("{}", c as char);
        }
    }
    if c.reg.c == 0x02 {
        print!("{}", c.reg.e as char);
    }
}
