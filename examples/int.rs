use std::{error::Error, process};
use zilog_z80::{bus::FlatBus, cpu::CPU, dasm::dasm};

fn main() {
    if let Err(e) = load_execute() {
        println!("{}", e);
        process::exit(1);
    }
}

// Traces the execution of tests/int_im2.asm: the program spins on CP B / JP NZ until an
// IM 2 interrupt makes its service routine load A (0x0F) into B, then returns to 0x0000.
fn load_execute() -> Result<(), Box<dyn Error>> {
    let mut b = FlatBus::new(0xFFFF);
    let mut c = CPU::new();

    // Loads assembled program into memory
    b.load_bin("bin/int_im2.bin", 0)?;

    // LD SP,0xFF00 / LD A,0x01 / LD I,A / LD A,0x0F / JP start / IM 2 / EI / CP B / JP NZ,@loop
    for _ in 0..9 {
        step(&mut c, &mut b);
    }

    println!("--- INT requested, vector 0x02 ---");
    c.int_request(0x02);

    loop {
        step(&mut c, &mut b);
        if c.reg.pc == 0x0000 {
            break;
        }
    }
    println!("--- returned to 0x0000, B = {:#04X} ---", c.reg.b);
    Ok(())
}

// Disassembles the instruction about to run, executes it and reports the cycles it took.
// An interrupt acknowledge consumes a whole execute() call without running the instruction
// at PC, so it is detected (the pending request is cleared) and labelled as such.
fn step(c: &mut CPU, b: &mut FlatBus) {
    let pc = c.reg.pc;
    let (instr, _) = dasm(b, pc);
    let was_pending = c.has_pending_int();
    let cycles = c.execute(b);
    let acknowledged = was_pending && !c.has_pending_int();
    println!(
        "{:#06X}  {:<30} {:>2} cycles   A:{:02X} B:{:02X} SP:{:04X} IFF1:{}{}",
        pc,
        if acknowledged { String::new() } else { instr },
        cycles,
        c.reg.a,
        c.reg.b,
        c.reg.sp,
        c.iff1() as u8,
        if acknowledged {
            format!("   <- INT acknowledged, jumping to {:#06X}", c.reg.pc)
        } else {
            String::new()
        }
    );
}
