use std::{ error::Error, process, thread };
use zilog_z80::cpu::CPU;

fn main() {
    if let Err(e) = load_execute() {
        println!("{}", e);
        process::exit(1);
    }
}

fn load_execute() -> Result<(), Box<dyn Error>> {
    let mut c = CPU::new();
    c.debug.io = false;
    c.debug.instr_in = false;

    // Loads assembled program into memory
    c.bus.load_bin("bin/out_a.bin", 0)?;

    // io.0 is the sender, io.1 is the receiver. Used to send / receive a (device, data) tuple to / from a peripheral.
    let io_receiver1 = c.io.1.clone();

    // In this example periph is the entry function that simulates a peripheral. It runs in a separate thread.
    thread::spawn(move || {
        periph(io_receiver1);
    });

    // A basic program which waits a moment then sends the 0xBB byte to the 0x07 peripheral
    loop {
        c.execute_slice();
        if c.debug.opcode { print!("{}\n", c.debug.string); }
        if c.reg.pc == 0x0000 { break }
    }
    Ok(())
}

// Demonstration peripheral 0x07 listens data sent b the CPU
fn periph(rx: crossbeam_channel::Receiver<(u8, u8)>) {
    loop {
        if let Ok((device, data)) = rx.try_recv() {
            if device == 0x07 { println!("The 0x07 peripheral received {:#04X} from the CPU", data) }
        }
    }
}
