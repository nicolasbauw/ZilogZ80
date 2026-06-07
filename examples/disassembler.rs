use std::{env, error::Error, process};
use zilog_z80::bus::Bus;

fn main() {
    if let Err(e) = load_disassemble() {
        println!("{}", e);
        process::exit(1);
    }
}

fn load_disassemble() -> Result<(), Box<dyn Error>> {
    let a: Vec<String> = env::args().collect();
    let mut b = Bus::new(0xFFFF);
    let mut address: u16 = 0;
    // Loads assembled program into memory
    let size = b.load_bin(&a[1], 0)?;

    while (address as usize) < size {
        let d = b.dasm(address);
        println!("{}", d.0);
        address += (d.1) as u16;
    }
    Ok(())
}
