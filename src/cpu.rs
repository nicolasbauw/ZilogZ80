use crate::registers::Registers;
use crate::memory::AddressBus;
use crate::bit;
use crate::cycles::{ CYCLES, CYCLES_CB, CYCLES_ED, CYCLES_DD_FD };
use std::{time::Duration, time::SystemTime};

pub struct CPU {
    pub reg: Registers,
    pub alt: Registers,
    pub bus: AddressBus,
    halt: bool,
    pub debug: Debug,
    int: Option<u8>,
    nmi: bool,
    im: u8,
    iff1: bool,
    iff2: bool,
    slice_duration: u32,
    /// Defaults to 35000 cycles per 16ms slice (2.1 Mhz).
    /// cycles = clock speed in Hz / required frames-per-second
    slice_max_cycles: u32,
    slice_current_cycles: u32,
    slice_start_time: SystemTime,
    pub io: (crossbeam_channel::Sender<(u8, u8)>, crossbeam_channel::Receiver<(u8, u8)>)
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            reg: Registers::new(),
            alt: Registers::new(),
            bus: AddressBus::new(),
            halt: false,
            debug: Debug::new(),
            int: None,
            nmi: false,
            im: 0,
            iff1: false,
            iff2: false,
            slice_duration: 16,
            slice_max_cycles: 35000,
            slice_current_cycles: 0,
            slice_start_time: SystemTime::now(),
            io: crossbeam_channel::unbounded(),
        }
    }

    pub fn int_request(&mut self, byte: u8) {
        self.int = Some(byte);
    }

    pub fn nmi_request(&mut self) {
        self.nmi = true;
    }

    pub fn flags(&self) -> u8 {
        self.reg.flags.to_byte()
    }

    fn ldi(&mut self) {
        let bc = self.reg.get_bc();
        let de = self.reg.get_de();
        let hl = self.reg.get_hl();
        self.bus.write_byte(de, self.bus.read_byte(hl));
        self.reg.set_de(de.wrapping_add(1));
        self.reg.set_hl(hl.wrapping_add(1));
        self.reg.set_bc(bc.wrapping_sub(1));
    }

    fn ldd(&mut self) {
        let bc = self.reg.get_bc();
        let de = self.reg.get_de();
        let hl = self.reg.get_hl();
        self.bus.write_byte(de, self.bus.read_byte(hl));
        self.reg.set_de(de.wrapping_sub(1));
        self.reg.set_hl(hl.wrapping_sub(1));
        self.reg.set_bc(bc.wrapping_sub(1));
    }

    // Returns A - (HL)
    fn cpi(&mut self) {
        let bc = self.reg.get_bc();
        let hl = self.reg.get_hl();
        let h = self.bus.read_byte(hl);
        let r = self.reg.a.wrapping_sub(h);
        
        self.reg.set_hl(hl.wrapping_add(1));
        self.reg.set_bc(bc.wrapping_sub(1));
        
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = self.reg.a == h;
        self.reg.flags.h = (self.reg.a as i8 & 0x0F) < (h as i8 & 0x0F);
        self.reg.flags.p = self.reg.get_bc() != 0;
        self.reg.flags.n = true;
    }

    // Returns A - (HL)
    fn cpd(&mut self) {
        let bc = self.reg.get_bc();
        let hl = self.reg.get_hl();
        let h = self.bus.read_byte(hl);
        let r = self.reg.a.wrapping_sub(h);

        self.reg.set_hl(hl.wrapping_sub(1));
        self.reg.set_bc(bc.wrapping_sub(1));

        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = self.reg.a == h;
        self.reg.flags.h = (self.reg.a as i8 & 0x0F) < (h as i8 & 0x0F);
        self.reg.flags.p = self.reg.get_bc() != 0;
        self.reg.flags.n = true;
    }

    // ADD A,r
    fn add(&mut self, n: u8)  {
        let a = self.reg.a;
        let r = a.wrapping_add(n);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.p = check_add_overflow(self.reg.a, n);
        self.reg.flags.h = (a & 0x0f) + (n & 0x0f) > 0x0f;
        self.reg.flags.c = u16::from(a) + u16::from(n) > 0xff;
        self.reg.flags.n = false;
        self.reg.a = r;
    }

    // ADD A,s : ADD with carry
    fn adc(&mut self, n: u8)  {
        let c: u8 = match self.reg.flags.c {
            false => 0,
            true => 1,
        };
        let a = self.reg.a;
        let r = a.wrapping_add(n).wrapping_add(c);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.p = check_add_overflow(self.reg.a, n.wrapping_add(c));
        self.reg.flags.h = (a & 0x0f) + (n & 0x0f) + c > 0x0f;
        self.reg.flags.c = u16::from(a) + u16::from(n) + u16::from(c) > 0xff;
        self.reg.flags.n = false;
        self.reg.a = r;
    }

    // SUB s
    fn sub(&mut self, n: u8)  {
        let a = self.reg.a;
        let r = a.wrapping_sub(n);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.p = check_sub_overflow(self.reg.a, n);
        self.reg.flags.h = (a as i8 & 0x0f) < (n as i8 & 0x0f);
        self.reg.flags.c = u16::from(a) < u16::from(n);
        self.reg.flags.n = true;
        self.reg.a = r;
    }

    // SBC s
    fn sbc(&mut self, n: u8)  {
        let c: u8 = match self.reg.flags.c {
            false => 0,
            true => 1,
        };
        let a = self.reg.a;
        let r = a.wrapping_sub(n.wrapping_add(c));
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.p = check_sub_overflow(self.reg.a, n.wrapping_add(c));
        self.reg.flags.h = (a as i8 & 0x0f) < (n as i8 & 0x0f).wrapping_add(c as i8);
        self.reg.flags.c = u16::from(a) < (u16::from(n) + u16::from(c));
        self.reg.flags.n = true;
        self.reg.a = r;
    }

    // Logical AND
    fn and(&mut self, n: u8) {
        let r = self.reg.a & n;
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        self.reg.flags.h = true;
        self.reg.flags.c = false;
        self.reg.flags.n = false;
        self.reg.a = r;
    }

    // Logical OR
    fn or(&mut self, n: u8) {
        let r = self.reg.a | n;
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        self.reg.flags.h = false;
        self.reg.flags.c = false;
        self.reg.flags.n = false;
        self.reg.a = r;
    }
    
    // Logical exclusive-OR
    fn xor(&mut self, n: u8) {
        let a = self.reg.a;
        let r = a ^ n;
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        self.reg.flags.h = false;
        self.reg.flags.c = false;
        self.reg.flags.n = false;
        self.reg.a = r;
    }

    // Comparison with accumulator
    fn cp(&mut self, n: u8) {
        let r = self.reg.a;
        self.sub(n);
        self.reg.a = r;
    }

    // Increment
    fn inc(&mut self, n: u8) -> u8 {
        let r = n.wrapping_add(1);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.p = n == 0x7F;
        self.reg.flags.h = (n & 0x0f) + 0x01 > 0x0f;
        self.reg.flags.n = false;
        r
    }

    // Decrement
    fn dec(&mut self, n: u8) -> u8 {
        let r = n.wrapping_sub(1);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.p = n == 0x80;
        self.reg.flags.h = ((n & 0x0f) as i8) < 1;
        self.reg.flags.n = true;
        r
    }

    // Decimal adjust accumulator
    // Function reworked using Rui F Ribeiro implementation (https://stackoverflow.com/questions/8119577/z80-daa-instruction)
    fn daa(&mut self) {
        let mut t = 0;
        let lsb = self.reg.a & 0x0F;
        
        if self.reg.flags.h || (lsb > 9) {
            t += 1;
        }

        if self.reg.flags.c || ( self.reg.a > 0x99) {
            t += 2;
            self.reg.flags.c = true;
        }

        if self.reg.flags.n && !self.reg.flags.h { self.reg.flags.h = false }
        else
        {
            if self.reg.flags.n && self.reg.flags.h { self.reg.flags.h = lsb < 6; }
            else { self.reg.flags.h = lsb >= 0x0A; }
        }

        match t {
            1 => {
                let r =  match self.reg.flags.n {
                    true => 0xFA,   // -6
                    false => 0x06,  // 6
                };
                self.reg.a = self.reg.a.wrapping_add(r);
            }
                
            2 => {
                let r = match self.reg.flags.n {
                    true => 0xA0,   // -0x60
                    false => 0x60,  // 0x60
                };
                self.reg.a = self.reg.a.wrapping_add(r);
            }
            
            3 => {
                let r = match self.reg.flags.n {
                    true => 0x9A,   // -0x66
                    false => 0x66,  // 0x66
                };
                self.reg.a = self.reg.a.wrapping_add(r);
            }

            _ => {},
        }

        self.reg.flags.z = self.reg.a == 0x00;
        self.reg.flags.s = bit::get(self.reg.a, 7);
        self.reg.flags.p = self.reg.a.count_ones() & 0x01 == 0x00;
    }

    // NEG
    fn neg(&mut self) {
        let t = !self.reg.a;
        let r = t.wrapping_add(1);
        self.reg.flags.p = self.reg.a == 0x80;
        self.reg.flags.c = self.reg.a != 0;
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.h = 0 < (self.reg.a & 0x0F);
        self.reg.flags.n = true;
        self.reg.a = r;
    }

    // 16 bits add
    fn add_16(&mut self, n1: u16, n2: u16) -> u16 {
        let r = n1.wrapping_add(n2);
        self.reg.flags.c = u32::from(n1) + u32::from(n2) > 0xffff;
        self.reg.flags.h = (n1 & 0x0FFF) + (n2 & 0x0FFF) > 0x0FFF;
        self.reg.flags.n = false;
        r
    }

    // Register pair addition with carry
    fn adc_16(&mut self, n: u16) {
        let c: u16 = match self.reg.flags.c {
            false => 0,
            true => 1,
        };
        let h = self.reg.get_hl();
        let r = h.wrapping_add(n).wrapping_add(c);
        self.reg.set_hl(r);
        self.reg.flags.s = (r as i16) < 0;
        self.reg.flags.z = r == 0x00;
        self.reg.flags.c = u32::from(h) + u32::from(n) + c as u32 > 0xffff;
        self.reg.flags.h = (h & 0x0FFF) + (n & 0x0FFF) + c > 0x0FFF;
        self.reg.flags.n = false;
        self.reg.flags.p = {
            let r = (h as i16).overflowing_add((n + c) as i16);
            r.1
        }
    }

    // Register pair substraction with carry
    fn sbc_16(&mut self, n: u16)  {
        let c: u16 = match self.reg.flags.c {
            false => 0,
            true => 1,
        };
        let h = self.reg.get_hl();
        let r = h.wrapping_sub(n).wrapping_sub(c);
        self.reg.set_hl(r);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i16) < 0;
        self.reg.flags.h = (h  & 0x0fff) < (n & 0x0fff)  + c;
        self.reg.flags.c = u16::from(h) < u16::from(n) + u16::from(c);
        self.reg.flags.n = true;
        self.reg.flags.p = {
            let r = (h as i16).overflowing_sub((n + c)as i16);
            r.1
        }
    }

    // Rotate Accumulator left
    fn rlca(&mut self) {
        self.reg.flags.c = bit::get(self.reg.a, 7);
        let r = (self.reg.a << 1) | u8::from(self.reg.flags.c);
        self.reg.flags.c = bit::get(self.reg.a, 7);
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        self.reg.a = r;
    }

    // Rotate left
    fn rlc(&mut self, n: u8) -> u8 {
        self.reg.flags.c = bit::get(n, 7);
        let r = (n << 1) | u8::from(self.reg.flags.c);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        r
    }

    // Rotate Accumulator right
    fn rrca(&mut self) {
        self.reg.flags.c = bit::get(self.reg.a, 0);
        let r = if self.reg.flags.c {0x80 | (self.reg.a >> 1) } else { self.reg.a >> 1 };
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        self.reg.a = r;
    }

    // Rotate right
    fn rrc(&mut self, n: u8) -> u8 {
        self.reg.flags.c = bit::get(n, 0);
        let r = if self.reg.flags.c {0x80 | (n >> 1) } else { n >> 1 };
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        self.reg.flags.n = false;
        r
    }

    // Rotate Accumulator left through carry
    fn rla(&mut self) {
        let c = self.reg.flags.c;
        self.reg.flags.c = bit::get(self.reg.a, 7);
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        let r = match c {
            true => (self.reg.a << 1) | 0x01,
            false => self.reg.a << 1
        };
        self.reg.a = r;
    }

    // Rotate left through carry
    fn rl(&mut self, n: u8) -> u8 {
        let c = self.reg.flags.c;
        self.reg.flags.c = bit::get(n, 7);
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        let r = match c {
            true => (n << 1) | 0x01,
            false => n << 1
        };
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        r
    }
    
    // Rotate Accumulator right through carry
    fn rra(&mut self) {
        let c = self.reg.flags.c;
        self.reg.flags.c = bit::get(self.reg.a, 0);
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        let r = match c {
            true => (self.reg.a >> 1) | 0x80,
            false => self.reg.a >> 1
        };
        self.reg.a = r;
    }

    // Rotate right through carry
    fn rr(&mut self, n: u8) -> u8 {
        let c = self.reg.flags.c;
        self.reg.flags.c = bit::get(n, 0);
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        let r = match c {
            true => (n >> 1) | 0x80,
            false => n >> 1
        };
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        r
    }

    // Arithmetic shift left
    fn sla(&mut self, n: u8) -> u8 {
        let r = n << 1;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0x00;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        self.reg.flags.n = false;
        self.reg.flags.c = bit::get(n, 7);
        r
    }

    // Logical shift left
    fn sll(&mut self, n: u8) -> u8 {
        let r = (n << 1) | 0x01;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0x00;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        self.reg.flags.n = false;
        self.reg.flags.c = bit::get(n, 7);
        r
    }

    // Arithmetic shift right
    fn sra(&mut self, n: u8) -> u8 {
        // https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators
        // *** Arithmetic right shift on signed integer types, logical right shift on unsigned integer types.
        let r = ((n as i8) >> 1) as u8;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0x00;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        self.reg.flags.n = false;
        self.reg.flags.c = bit::get(n, 0);
        r
    }

    // Logical shift right
    fn srl(&mut self, n: u8) -> u8 {
        // https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators
        // *** Arithmetic right shift on signed integer types, logical right shift on unsigned integer types.
        let r = n >> 1;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0x00;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        self.reg.flags.n = false;
        self.reg.flags.c = bit::get(n, 0);
        r
    }

    // Bit test
    fn bit(&mut self, operand: u8) {
        let bit = ((operand & 0x38) >> 3) as usize;
        let register = operand & 0x07;
        let r = match register {
            0 => bit::get(self.reg.b, bit),
            1 => bit::get(self.reg.c, bit),
            2 => bit::get(self.reg.d, bit),
            3 => bit::get(self.reg.e, bit),
            4 => bit::get(self.reg.h, bit),
            5 => bit::get(self.reg.l, bit),
            6 => bit::get(self.bus.read_byte(self.reg.get_hl()), bit),
            7 => bit::get(self.reg.a, bit),
            _ => false
        };
        self.reg.flags.z = r == false;
        self.reg.flags.h = true;
        self.reg.flags.n = false;
    }

    // Bit set
    fn set(&mut self, operand: u8) {
        let bit = ((operand & 0x38) >> 3) as usize;
        let register = operand & 0x07;
        match register {
            0 => self.reg.b = bit::set(self.reg.b, bit),
            1 => self.reg.c = bit::set(self.reg.c, bit),
            2 => self.reg.d = bit::set(self.reg.d, bit),
            3 => self.reg.e = bit::set(self.reg.e, bit),
            4 => self.reg.h = bit::set(self.reg.h, bit),
            5 => self.reg.l = bit::set(self.reg.l, bit),
            6 => self.bus.write_byte(self.reg.get_hl(), bit::set(self.bus.read_byte(self.reg.get_hl()), bit)),
            7 => self.reg.a = bit::set(self.reg.a, bit),
            _ => {}
        };
    }

    // Bit reset
    fn reset(&mut self, operand: u8) {
        let bit = ((operand & 0x38) >> 3) as usize;
        let register = operand & 0x07;
        match register {
            0 => self.reg.b = bit::reset(self.reg.b, bit),
            1 => self.reg.c = bit::reset(self.reg.c, bit),
            2 => self.reg.d = bit::reset(self.reg.d, bit),
            3 => self.reg.e = bit::reset(self.reg.e, bit),
            4 => self.reg.h = bit::reset(self.reg.h, bit),
            5 => self.reg.l = bit::reset(self.reg.l, bit),
            6 => self.bus.write_byte(self.reg.get_hl(), bit::reset(self.bus.read_byte(self.reg.get_hl()), bit)),
            7 => self.reg.a = bit::reset(self.reg.a, bit),
            _ => {}
        };
    }

    // call stack push
    fn call_stack_push(&mut self) {
        self.reg.sp = self.reg.sp.wrapping_sub(2);
        self.bus.write_word(self.reg.sp , self.reg.pc.wrapping_add(3));
    }

    // call stack pop
    fn call_stack_pop(&mut self) {
        self.reg.pc = self.bus.read_word(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(2);
    }

    // interrupt stack push
    fn interrupt_stack_push(&mut self) {
        self.reg.sp = self.reg.sp.wrapping_sub(2);
        self.bus.write_word(self.reg.sp , self.reg.pc);
    }

    // IN r,(C)
    fn inrc(&mut self) -> u8 {
        let r = self.get_io(self.reg.c);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        r
    }

    // IN : from peripherals to CPU
    fn get_io(&mut self, port: u8) -> u8 {
        if let Ok((device, data)) = self.io.1.try_recv() {
            if self.debug.io { println!("Message {:#04X} from device {:#04X}", data, device) }
            if device == port { return data }
        }
        return 0
    }

    // OUT : from CPU to peripherals
    fn set_io(&mut self, port: u8, data: u8) {
        if self.debug.io { println!("Message {:#04X} sent to device {:#04X}", data, port) }
        self.io.0.send((port,data)).unwrap();
    }

    pub fn execute(&mut self) -> u32 {
        if self.halt { return 4 };

        // Non maskable interrupt requested ?
        if self.nmi {
            self.iff2 = self.iff1;
            self.iff1 = false;
            self.interrupt_stack_push();
            self.reg.pc = 0x0066;
            self.nmi = false;
        }

        // Interrupt requested in interrupt mode 1 ? Restart at address 0038h (opcode 0xFF)
        if self.iff1 && self.int.is_some() && self.im == 1 { self.int = Some(0xFF) };

        // Interrupt requested in interrupt mode 2 ? Push PC onto the stack, build jump address and jump to that address
        if self.iff1 && self.int.is_some() && self.im == 2 {
            self.interrupt_stack_push();
            let addr = ((self.reg.i as u16) << 8) | (self.int.unwrap() as u16);
            self.reg.pc = self.bus.read_word(addr);
            self.int = None;
        };

        // We retrieve the opcode, wether it comes from an interrupt request or normal fetch
        let opcode = match self.iff1 {
            false => self.bus.read_byte(self.reg.pc),
            // interrupts enabled : is there a pending interrupt ?
            true => match self.int {
                None =>  self.bus.read_byte(self.reg.pc),
                Some(o) => o as u8,
            },
        };

        let cycles = match opcode {
            0xDD | 0xFD | 0xED | 0xCB => self.execute_2bytes(),
            _ => self.execute_1byte(opcode),
        };

        self.int = None;
        cycles

    }

    /// Fetches and executes one instruction from (pc), limiting speed to 2,1 Mhz by default. Returns the number of consumed clock cycles.
    pub fn execute_slice(&mut self) -> u32 {
        if self.slice_current_cycles > self.slice_max_cycles {
            self.slice_current_cycles = 0;
            // d = time taken to execute the slice_max_cycles
            if let Ok(d) = self.slice_start_time.elapsed() {
                let sleep_time = self.slice_duration.saturating_sub(d.as_millis() as u32);
                /*println!("Execution time : {:?}", d);
                println!("Sleep time : {:?}", sleep_time);*/

                #[cfg(windows)]
                spin_sleep::sleep(Duration::from_millis(u64::from(sleep_time)));

                #[cfg(not(windows))]
                std::thread::sleep(Duration::from_millis(u64::from(sleep_time)));

                self.slice_start_time = SystemTime::now();
            }
        }
        let cycles = self.execute();
        self.slice_current_cycles += cycles;
        cycles
    }

    // DDCB FDCB
    fn execute_4bytes(&mut self) -> u32 {
        let opcode = self.bus.read_le_dword(self.reg.pc);
        let cycles;

        match opcode & 0xFFFF00FF {
            0xDDCB0006 => {                                                           // RLC (IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.rlc(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_ix() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.rlc(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xFDCB0006 => {                                                           // RLC (IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.rlc(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_iy() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.rlc(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xDDCB0016 => {                                                           // RL (IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                    if bit::get(displacement, 7) {
                        let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                        let d = self.bus.read_byte(m);
                        let r = self.rl(d);
                        self.bus.write_byte(m, r);
                    }
                    else {
                        let m = self.reg.get_ix() + ( displacement as u16 );
                        let d = self.bus.read_byte(m);
                        let r = self.rl(d);
                        self.bus.write_byte(m, r);
                    }
                    cycles = 23;
            },

            0xFDCB0016 => {                                                           // RL (IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                    if bit::get(displacement, 7) {
                        let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                        let d = self.bus.read_byte(m);
                        let r = self.rl(d);
                        self.bus.write_byte(m, r);
                    }
                    else {
                        let m = self.reg.get_iy() + ( displacement as u16 );
                        let d = self.bus.read_byte(m);
                        let r = self.rl(d);
                        self.bus.write_byte(m, r);
                    }
                    cycles = 23;
            },

            0xDDCB000E => {                                                           // RRC (IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.rrc(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_ix() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.rrc(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xFDCB000E => {                                                           // RRC (IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.rrc(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_iy() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.rrc(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xDDCB001E => {                                                           // RR (IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.rr(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_ix() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.rr(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xFDCB001E => {                                                           // RR (IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.rr(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_iy() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.rr(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xDDCB0026 => {                                                           // SLA (IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.sla(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_ix() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.sla(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xFDCB0026 => {                                                           // SLA (IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.sla(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_iy() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.sla(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xDDCB002E => {                                                           // SRA (IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.sra(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_ix() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.sra(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xFDCB002E => {                                                           // SRA (IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.sra(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_iy() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.sra(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xDDCB003E => {                                                           // SRL (IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.srl(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_ix() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.srl(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xFDCB003E => {                                                           // SRL (IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.srl(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_iy() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.srl(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xDDCB0046 | 0xDDCB004E | 0xDDCB0056 |
            0xDDCB005E | 0xDDCB0066 | 0xDDCB006E |
            0xDDCB0076 | 0xDDCB007E => {                                                           // BIT b,(IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                let operand = self.bus.read_byte(self.reg.pc + 3);
                let bit = ((operand & 0x38) >> 3) as usize;
                if bit::get(displacement, 7) {
                    let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = bit::get(d, bit);
                    self.reg.flags.z = r == false;
                    self.reg.flags.h = true;
                    self.reg.flags.n = false;
                    
                }
                else {
                    let m = self.reg.get_ix() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = bit::get(d, bit);
                    self.reg.flags.z = r == false;
                    self.reg.flags.h = true;
                    self.reg.flags.n = false;
                }
                cycles = 20;
            },

            0xFDCB0046 | 0xFDCB004E | 0xFDCB0056 |
            0xFDCB005E | 0xFDCB0066 | 0xFDCB006E |
            0xFDCB0076 | 0xFDCB007E => {                                                           // BIT b,(IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                let operand = self.bus.read_byte(self.reg.pc + 3);
                let bit = ((operand & 0x38) >> 3) as usize;
                if bit::get(displacement, 7) {
                    let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = bit::get(d, bit);
                    self.reg.flags.z = r == false;
                    self.reg.flags.h = true;
                    self.reg.flags.n = false;
                    
                }
                else {
                    let m = self.reg.get_iy() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = bit::get(d, bit);
                    self.reg.flags.z = r == false;
                    self.reg.flags.h = true;
                    self.reg.flags.n = false;
                }
                cycles = 20;
            },

            0xDDCB00C6 | 0xDDCB00CE | 0xDDCB00D6 |
            0xDDCB00DE | 0xDDCB00E6 | 0xDDCB00EE |
            0xDDCB00F6 | 0xDDCB00FE => {                                                           // SET b,(IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                let operand = self.bus.read_byte(self.reg.pc + 3);
                let bit = ((operand & 0x38) >> 3) as usize;
                if bit::get(displacement, 7) {
                    let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = bit::set(d, bit);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_ix() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = bit::set(d, bit);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xFDCB00C6 | 0xFDCB00CE | 0xFDCB00D6 |
            0xFDCB00DE | 0xFDCB00E6 | 0xFDCB00EE |
            0xFDCB00F6 | 0xFDCB00FE => {                                                           // SET b,(IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                let operand = self.bus.read_byte(self.reg.pc + 3);
                let bit = ((operand & 0x38) >> 3) as usize;
                if bit::get(displacement, 7) {
                    let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = bit::set(d, bit);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_iy() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = bit::set(d, bit);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xDDCB0086 | 0xDDCB008E | 0xDDCB0096 |
            0xDDCB009E | 0xDDCB00A6 | 0xDDCB00AE |
            0xDDCB00B6 | 0xDDCB00BE => {                                                           // RES b,(IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                let operand = self.bus.read_byte(self.reg.pc + 3);
                let bit = ((operand & 0x38) >> 3) as usize;
                if bit::get(displacement, 7) {
                    let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = bit::reset(d, bit);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_ix() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = bit::reset(d, bit);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            0xFDCB0086 | 0xFDCB008E | 0xFDCB0096 |
            0xFDCB009E | 0xFDCB00A6 | 0xFDCB00AE |
            0xFDCB00B6 | 0xFDCB00BE => {                                                           // RES b,(IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                let operand = self.bus.read_byte(self.reg.pc + 3);
                let bit = ((operand & 0x38) >> 3) as usize;
                if bit::get(displacement, 7) {
                    let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = bit::reset(d, bit);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_iy() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = bit::reset(d, bit);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            // Undocumented instructions
            // SLL (IX+d)
            0xDDCB0036 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.sll(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_ix() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.sll(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            // SLL (IY+d)
            0xFDCB0036 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.sll(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_iy() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.sll(d);
                    self.bus.write_byte(m, r);
                }
                cycles = 23;
            },

            _ => {
                if self.debug.unknw_instr { self.debug.string = format!("{:#10X}", opcode) };
                cycles = 0xFF;
                }
        }
        self.reg.pc += 4;
        if self.debug.opcode == true { self.debug.string = format!("{:#10X}", opcode) }
        cycles
    }

    fn execute_2bytes(&mut self) -> u32 {
        let opcode = self.bus.read_le_word(self.reg.pc);
        let mut cycles = match opcode & 0xFF00 {
                0xDD00 | 0xFD00 => CYCLES_DD_FD[(opcode & 0x00FF) as usize].into(),
                0xED00 => CYCLES_ED[(opcode & 0x00FF) as usize].into(),
                0xCB00 => CYCLES_CB[(opcode & 0x00FF) as usize].into(),
                _ => 0
        };

        match opcode {
            // 4 bytes instructions
            0xDDCB | 0xFDCB => return self.execute_4bytes(),

            // 8-Bit Load Group
            // LD r,(IX+d)
            0xDD46 => {                                                             // LD B,(IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.b = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.b = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 )) }
            },
            0xDD4E => {                                                             // LD C,(IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.c = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.c = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 )) }
            },
            0xDD56 => {                                                             // LD D,(IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.d = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.d = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 )) }
            },
            0xDD5E => {                                                             // LD E,(IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.e = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.e = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 )) }
            },
            0xDD66 => {                                                             // LD H,(IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.h = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.h = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 )) }
            },
            0xDD6E => {                                                             // LD L,(IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.l = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.l = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 )) }
            },
            0xDD7E => {                                                             // LD A,(IX+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.a = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.a = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 )) }
            },

            // LD r,(IY+d)
            0xFD46 => {                                                             // LD B,(IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.b = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.b = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 )) }
            },
            0xFD4E => {                                                             // LD C,(IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.c = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.c = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 )) }
            },
            0xFD56 => {                                                             // LD D,(IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.d = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.d = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 )) }
            },
            0xFD5E => {                                                             // LD E,(IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.e = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.e = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 )) }
            },
            0xFD66 => {                                                             // LD H,(IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.h = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.h = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 )) }
            },
            0xFD6E => {                                                             // LD L,(IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.l = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.l = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 )) }
            },
            0xFD7E => {                                                             // LD A,(IY+d)
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.reg.a = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 )) }
                else { self.reg.a = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 )) }
            },

            // LD (IX+d),r
            0xDD70 => {                                                             // LD (IX+d),B
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ), self.reg.b) }
                else { self.bus.write_byte(self.reg.get_ix() + ( displacement as u16 ), self.reg.b) }
            },
            0xDD71 => {                                                             // LD (IX+d),C
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ), self.reg.c) }
                else { self.bus.write_byte(self.reg.get_ix() + ( displacement as u16 ), self.reg.c) }
            },
            0xDD72 => {                                                             // LD (IX+d),D
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ), self.reg.d) }
                else { self.bus.write_byte(self.reg.get_ix() + ( displacement as u16 ), self.reg.d) }
            },
            0xDD73 => {                                                             // LD (IX+d),E
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ), self.reg.e) }
                else { self.bus.write_byte(self.reg.get_ix() + ( displacement as u16 ), self.reg.e) }
            },
            0xDD74 => {                                                             // LD (IX+d),H
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ), self.reg.h) }
                else { self.bus.write_byte(self.reg.get_ix() + ( displacement as u16 ), self.reg.h) }
            },
            0xDD75 => {                                                             // LD (IX+d),L
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ), self.reg.l) }
                else { self.bus.write_byte(self.reg.get_ix() + ( displacement as u16 ), self.reg.l) }
            },
            0xDD77 => {                                                             // LD (IX+d),A
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ), self.reg.a) }
                else { self.bus.write_byte(self.reg.get_ix() + ( displacement as u16 ), self.reg.a) }
            },

            // LD (IY+d),r
            0xFD70 => {                                                             // LD (IY+d),B
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ), self.reg.b) }
                else { self.bus.write_byte(self.reg.get_iy() + ( displacement as u16 ), self.reg.b) }
            },
            0xFD71 => {                                                             // LD (IY+d),C
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ), self.reg.c) }
                else { self.bus.write_byte(self.reg.get_iy() + ( displacement as u16 ), self.reg.c) }
            },
            0xFD72 => {                                                             // LD (IY+d),D
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ), self.reg.d) }
                else { self.bus.write_byte(self.reg.get_iy() + ( displacement as u16 ), self.reg.d) }
            },
            0xFD73 => {                                                             // LD (IY+d),E
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ), self.reg.e) }
                else { self.bus.write_byte(self.reg.get_iy() + ( displacement as u16 ), self.reg.e) }
            },
            0xFD74 => {                                                             // LD (IY+d),H
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ), self.reg.h) }
                else { self.bus.write_byte(self.reg.get_iy() + ( displacement as u16 ), self.reg.h) }
            },
            0xFD75 => {                                                             // LD (IY+d),L
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ), self.reg.l) }
                else { self.bus.write_byte(self.reg.get_iy() + ( displacement as u16 ), self.reg.l) }
            },
            0xFD77 => {                                                             // LD (IY+d),A
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ), self.reg.a) }
                else { self.bus.write_byte(self.reg.get_iy() + ( displacement as u16 ), self.reg.a) }
            },

            // LD (IX+d),n
            0xDD36 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                let data = self.bus.read_byte(self.reg.pc + 3);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ), data) }
                else { self.bus.write_byte(self.reg.get_ix() + ( displacement as u16 ), data) }
            }

            // LD IX,nn
            0xDD21 => {
                self.reg.set_ix(self.bus.read_word(self.reg.pc + 2));
            }


            // LD IY,nn
            0xFD21 => {
                self.reg.set_iy(self.bus.read_word(self.reg.pc + 2));
            }

            // LD (IY+d),n
            0xFD36 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                let data = self.bus.read_byte(self.reg.pc + 3);
                if bit::get(displacement, 7) { self.bus.write_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ), data) }
                else { self.bus.write_byte(self.reg.get_iy() + ( displacement as u16 ), data) }
            }

            // LD A,I
            0xED57 => {
                self.reg.a = self.reg.i;
                self.reg.flags.s = (self.reg.i as i8) < 0;
                self.reg.flags.z = self.reg.i == 0;
                self.reg.flags.h = false;
                self.reg.flags.p = self.iff2;
                self.reg.flags.n = false;
                // TODO :
                // If an interrupt occurs during execution of this instruction, the Parity flag contains a 0.
            },

            // LD A,R
            0xED5F => {
                self.reg.a = self.reg.r;
                self.reg.flags.s = (self.reg.r as i8) < 0;
                self.reg.flags.z = self.reg.r == 0;
                self.reg.flags.h = false;
                self.reg.flags.p = self.iff2;
                self.reg.flags.n = false;
                // TODO :
                // If an interrupt occurs during execution of this instruction, the Parity flag contains a 0.
            },

            // LD I,A
            0xED47 => self.reg.i = self.reg.a,

            // LD R,A
            0xED4F => self.reg.r = self.reg.a,

            // 16-Bit Load Group
            // LD dd,(nn)
            0xED4B => {                                                             // LD BC,(nn)
                let addr = self.bus.read_word(self.reg.pc +2);
                let d = self.bus.read_word(addr);
                self.reg.set_bc(d);
            },

            0xED5B => {                                                             // LD DE,(nn)
                let addr = self.bus.read_word(self.reg.pc +2);
                let d = self.bus.read_word(addr);
                self.reg.set_de(d);
            },

            0xED6B => {                                                             // LD HL,(nn)
                let addr = self.bus.read_word(self.reg.pc +2);
                let d = self.bus.read_word(addr);
                self.reg.set_hl(d);
            },

            0xED7B => {                                                             // LD SP,(nn)
                let addr = self.bus.read_word(self.reg.pc +2);
                let d = self.bus.read_word(addr);
                self.reg.sp = d;
            },

            // LD IX,(nn)
            0xDD2A => {
                let addr = self.bus.read_word(self.reg.pc +2);
                let d = self.bus.read_word(addr);
                self.reg.set_ix(d);
            },

            // LD IY,(nn)
            0xFD2A => {
                let addr = self.bus.read_word(self.reg.pc +2);
                let d = self.bus.read_word(addr);
                self.reg.set_iy(d);
            },

            // LD (nn),dd
            0xED43 => {                                                             // LD (nn),BC
                let addr = self.bus.read_word(self.reg.pc +2);
                self.bus.write_word(addr, self.reg.get_bc());
            },

            0xED53 => {                                                             // LD (nn),DE
                let addr = self.bus.read_word(self.reg.pc +2);
                self.bus.write_word(addr, self.reg.get_de());
            },

            0xED63 => {                                                             // LD (nn),HL
                let addr = self.bus.read_word(self.reg.pc +2);
                self.bus.write_word(addr, self.reg.get_hl());
            },

            0xED73 => {                                                             // LD (nn),SP
                let addr = self.bus.read_word(self.reg.pc +2);
                self.bus.write_word(addr, self.reg.sp);
            },

            // LD (nn),IX
            0xDD22 => {
                let addr = self.bus.read_word(self.reg.pc +2);
                self.bus.write_word(addr, self.reg.get_ix());
            },

            // LD (nn),IY
            0xFD22 => {
                let addr = self.bus.read_word(self.reg.pc +2);
                self.bus.write_word(addr, self.reg.get_iy());
            },

            // LD SP,IX
            0xDDF9 => self.reg.sp = self.reg.get_ix(),

            // LD SP,IY
            0xFDF9 => self.reg.sp = self.reg.get_iy(),

            // PUSH IX
            0xDDE5 => {
                self.reg.sp = self.reg.sp.wrapping_sub(2);
                self.bus.write_word(self.reg.sp, self.reg.get_ix());
            },

            // PUSH IY
            0xFDE5 => {
                self.reg.sp = self.reg.sp.wrapping_sub(2);
                self.bus.write_word(self.reg.sp, self.reg.get_iy());
            },

            // POP IX
            0xDDE1 => {
                self.reg.set_ix(self.bus.read_word(self.reg.sp));
                self.reg.sp = self.reg.sp.wrapping_add(2);
            },

            // POP IY
            0xFDE1 => {
                self.reg.set_iy(self.bus.read_word(self.reg.sp));
                self.reg.sp = self.reg.sp.wrapping_add(2);
            },

            // Exchange, Block Transfer, and Search Group
            // EX (SP),IX
            0xDDE3 => {
                let pointed_by_sp = self.bus.read_word(self.reg.sp);
                self.bus.write_word(self.reg.sp, self.reg.get_ix());
                self.reg.set_ix(pointed_by_sp);
            },

            // EX (SP),IY
            0xFDE3 => {
                let pointed_by_sp = self.bus.read_word(self.reg.sp);
                self.bus.write_word(self.reg.sp, self.reg.get_iy());
                self.reg.set_iy(pointed_by_sp);
            },

            // LDI
            0xEDA0 => {
                self.ldi();
                self.reg.flags.h = false;
                let bc = self.reg.get_bc();
                self.reg.flags.p = bc != 0;
                self.reg.flags.n = false;
            },

            // LDIR
            0xEDB0 => {
                // TODO : When the BC is set to 0 prior to instruction execution, the instruction loops through 64 KB.
                while self.reg.get_bc() !=0 {
                    self.ldi();
                    let bc = self.reg.get_bc();
                    self.reg.flags.h = false;
                    self.reg.flags.p = bc != 0;
                    self.reg.flags.n = false;
                    // TODO : return cycles * number of executions
                }
            },

            // LDD
            0xEDA8 => {
                self.ldd();
                self.reg.flags.h = false;
                let bc = self.reg.get_bc();
                self.reg.flags.p = bc != 0;
                self.reg.flags.n = false;
            },

            // LDDR
            0xEDB8 => {
                // TODO : When the BC is set to 0 prior to instruction execution, the instruction loops through 64 KB.
                while self.reg.get_bc() !=0 {
                    self.ldd();
                    let bc = self.reg.get_bc();
                    self.reg.flags.h = false;
                    self.reg.flags.p = bc != 0;
                    self.reg.flags.n = false;
                    // TODO : return cycles * number of executions
                }
            },

            // CPI
            0xEDA1 => self.cpi(),

            // CPIR
            0xEDB1 => {
                // TODO : When the BC is set to 0 prior to instruction execution, the instruction loops through 64 KB.
                while self.reg.get_bc() !=0 {
                    self.cpi();
                    if self.reg.flags.z { break }
                    // TODO : return cycles * number of executions
                }
            },

            // CPD
            0xEDA9 => self.cpd(),

            // CPDR
            0xEDB9 => {
                while self.reg.get_bc() !=0 {
                    self.cpd();
                    if self.reg.flags.z { break }
                    // TODO : return cycles * number of executions
                }
            },

            // 8-Bit Arithmetic Group
            // ADD A,(IX+d)
            0xDD86 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ));
                    self.add(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 ));
                    self.add(d);
                }
            },

            // ADD A,(IY+d)
            0xFD86 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ));
                    self.add(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 ));
                    self.add(d);
                }
            },

            // ADC A,(IX+d)
            0xDD8E => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ));
                    self.adc(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 ));
                    self.adc(d);
                }
            },

            // ADC A,(IY+d)
            0xFD8E => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ));
                    self.adc(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 ));
                    self.adc(d);
                }
            },

            // SUB (IX+d)
            0xDD96 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ));
                    self.sub(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 ));
                    self.sub(d);
                }
            },

            // SUB (IY+d)
            0xFD96 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ));
                    self.sub(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 ));
                    self.sub(d);
                }
            },

            // SBC (IX+d)
            0xDD9E => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ));
                    self.sbc(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 ));
                    self.sbc(d);
                }
            },

            // SBC (IY+d)
            0xFD9E => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ));
                    self.sbc(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 ));
                    self.sbc(d);
                }
            },

            // AND (IX+d)
            0xDDA6 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ));
                    self.and(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 ));
                    self.and(d);
                }
            },

            // AND (IY+d)
            0xFDA6 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ));
                    self.and(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 ));
                    self.and(d);
                }
            },

            // OR (IX+d)
            0xDDB6 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ));
                    self.or(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 ));
                    self.or(d);
                }
            },

            // OR (IY+d)
            0xFDB6 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ));
                    self.or(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 ));
                    self.or(d);
                }
            },

            // XOR (IX+d)
            0xDDAE => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ));
                    self.xor(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 ));
                    self.xor(d);
                }
            },

            // XOR (IY+d)
            0xFDAE => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ));
                    self.xor(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 ));
                    self.xor(d);
                }
            },

            // CP (IX+d)
            0xDDBE => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_ix() - ( signed_to_abs(displacement) as u16 ));
                    self.cp(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_ix() + ( displacement as u16 ));
                    self.cp(d);
                }
            },

            // CP (IY+d)
            0xFDBE => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let d = self.bus.read_byte(self.reg.get_iy() - ( signed_to_abs(displacement) as u16 ));
                    self.cp(d);
                }
                else {
                    let d = self.bus.read_byte(self.reg.get_iy() + ( displacement as u16 ));
                    self.cp(d);
                }
            },

            // INC (IX+d)
            0xDD34 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.inc(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_ix() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.inc(d);
                    self.bus.write_byte(m, r);
                }
            },

            // INC (IY+d)
            0xFD34 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.inc(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_iy() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.inc(d);
                    self.bus.write_byte(m, r);
                }
            },

            // DEC (IX+d)
            0xDD35 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_ix() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.dec(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_ix() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.dec(d);
                    self.bus.write_byte(m, r);
                }
            },

            // DEC (IY+d)
            0xFD35 => {
                let displacement = self.bus.read_byte(self.reg.pc + 2);
                if bit::get(displacement, 7) {
                    let m = self.reg.get_iy() - ( signed_to_abs(displacement) as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.dec(d);
                    self.bus.write_byte(m, r);
                }
                else {
                    let m = self.reg.get_iy() + ( displacement as u16 );
                    let d = self.bus.read_byte(m);
                    let r = self.dec(d);
                    self.bus.write_byte(m, r);
                }
            },

            // General-Purpose Arithmetic and CPU Control Groups
            // NEG
            0xED44 => {
                self.neg();
            },

            // RETI
            0xED4D => self.call_stack_pop(),

            // RETN
            0xED45 => {
                self.iff1 = self.iff2;
                self.call_stack_pop();
            },

            // Interrput modes
            0xED46 => self.im = 0,
            0xED56 => self.im = 1,
            0xED5E => self.im = 2,

            //16-Bit Arithmetic Group
            // ADC HL,ss
            0xED4A => {                                                             // ADC HL,BC
                let reg = self.reg.get_bc();
                self.adc_16(reg);
            },
            0xED5A => {                                                             // ADC HL,DE
                let reg = self.reg.get_de();
                self.adc_16(reg);
            },
            0xED6A => {                                                             // ADC HL,HL
                let reg = self.reg.get_hl();
                self.adc_16(reg);
            },
            0xED7A => {                                                             // ADC HL,SP
                let reg = self.reg.sp;
                self.adc_16(reg);
            },

            // SBC HL,ss
            0xED42 => {                                                             // SBC HL,BC
                let reg = self.reg.get_bc();
                self.sbc_16(reg);
            },
            0xED52 => {                                                             // SBC HL,DE
                let reg = self.reg.get_de();
                self.sbc_16(reg);
            },
            0xED62 => {                                                             // SBC HL,HL
                let reg = self.reg.get_hl();
                self.sbc_16(reg);
            },
            0xED72 => {                                                             // SBC HL,SP
                let reg = self.reg.sp;
                self.sbc_16(reg);
            },

            // ADD IX,pp
            0xDD09 => {                                                             // ADD IX,BC
                let reg = self.reg.get_bc();
                let r = self.add_16(self.reg.get_ix(), reg);
                self.reg.set_ix(r);
            },
            0xDD19 => {                                                             // ADD IX,DE
                let reg = self.reg.get_de();
                let r = self.add_16(self.reg.get_ix(), reg);
                self.reg.set_ix(r);
            },
            0xDD29 => {                                                             // ADD IX,IX
                let reg = self.reg.get_ix();
                let r = self.add_16(self.reg.get_ix(), reg);
                self.reg.set_ix(r);
            },
            0xDD39 => {                                                             // ADD IX,SP
                let reg = self.reg.sp;
                let r = self.add_16(self.reg.get_ix(), reg);
                self.reg.set_ix(r);
            },

            // ADD IY,pp
            0xFD09 => {                                                             // ADD IY,BC
                let reg = self.reg.get_bc();
                let r = self.add_16(self.reg.get_iy(), reg);
                self.reg.set_iy(r);
            },

            0xFD19 => {                                                             // ADD IY,DE
                let reg = self.reg.get_de();
                let r = self.add_16(self.reg.get_iy(), reg);
                self.reg.set_iy(r);
            },

            0xFD29 => {                                                             // ADD IY,IY
                let reg = self.reg.get_iy();
                let r = self.add_16(self.reg.get_iy(), reg);
                self.reg.set_iy(r);
            },

            0xFD39 => {                                                             // ADD IY,SP
                let reg = self.reg.sp;
                let r = self.add_16(self.reg.get_iy(), reg);
                self.reg.set_iy(r);
            },

            0xDD23 => {                                                             // INC IX
                let r = self.reg.get_ix().wrapping_add(1);
                self.reg.set_ix(r);
            },

            0xFD23 => {                                                             // INC IY
                let r = self.reg.get_iy().wrapping_add(1);
                self.reg.set_iy(r);
            },

            0xDD2B => {                                                             // DEC IX
                let r = self.reg.get_ix().wrapping_sub(1);
                self.reg.set_ix(r);
            },

            0xFD2B => {                                                             // DEC IY
                let r = self.reg.get_iy().wrapping_sub(1);
                self.reg.set_iy(r);
            },

            // Rotate and Shift Group
            // RLC r
            0xCB00 => {                                                             // RLC B
                let r = self.rlc(self.reg.b);
                self.reg.b = r;
            },

            0xCB01 => {                                                             // RLC C
                let r = self.rlc(self.reg.c);
                self.reg.c = r;
            },

            0xCB02 => {                                                             // RLC D
                let r = self.rlc(self.reg.d);
                self.reg.d = r;
            },

            0xCB03 => {                                                             // RLC E
                let r = self.rlc(self.reg.e);
                self.reg.e = r;
            },

            0xCB04 => {                                                             // RLC H
                let r = self.rlc(self.reg.h);
                self.reg.h = r;
            },

            0xCB05 => {                                                             // RLC L
                let r = self.rlc(self.reg.l);
                self.reg.l = r;
            },

            0xCB06 => {                                                             // RLC (HL)
                let addr = self.reg.get_hl();
                let r = self.rlc(self.bus.read_byte(addr));
                self.bus.write_byte(addr, r);
            },

            0xCB07 => {                                                             // RLC A
                let r = self.rlc(self.reg.a);
                self.reg.a = r;
            },

            // RL r
            0xCB10 => {                                                             // RL B
                let r = self.rl(self.reg.b);
                self.reg.b = r;
            },

            0xCB11 => {                                                             // RL C
                let r = self.rl(self.reg.c);
                self.reg.c = r;
            },

            0xCB12 => {                                                             // RL D
                let r = self.rl(self.reg.d);
                self.reg.d = r;
            },

            0xCB13 => {                                                             // RL E
                let r = self.rl(self.reg.e);
                self.reg.e = r;
            },

            0xCB14 => {                                                             // RL H
                let r = self.rl(self.reg.h);
                self.reg.h = r;
            },

            0xCB15 => {                                                             // RL L
                let r = self.rl(self.reg.l);
                self.reg.l = r;
            },

            0xCB16 => {                                                             // RL (HL)
                let addr = self.reg.get_hl();
                let r = self.rl(self.bus.read_byte(addr));
                self.bus.write_byte(addr, r);
            },

            0xCB17 => {                                                             // RL A
                let r = self.rl(self.reg.a);
                self.reg.a = r;
            },

            // RRC r
            0xCB08 => {                                                             // RRC B
                let r = self.rrc(self.reg.b);
                self.reg.b = r;
            },

            0xCB09 => {                                                             // RRC C
                let r = self.rrc(self.reg.c);
                self.reg.c = r;
            },

            0xCB0A => {                                                             // RRC D
                let r = self.rrc(self.reg.d);
                self.reg.d = r;
            },

            0xCB0B => {                                                             // RRC E
                let r = self.rrc(self.reg.e);
                self.reg.e = r;
            },

            0xCB0C => {                                                             // RRC H
                let r = self.rrc(self.reg.h);
                self.reg.h = r;
            },

            0xCB0D => {                                                             // RRC L
                let r = self.rrc(self.reg.l);
                self.reg.l = r;
            },

            0xCB0E => {                                                             // RR (HL)
                let addr = self.reg.get_hl();
                let r = self.rrc(self.bus.read_byte(addr));
                self.bus.write_byte(addr, r);
            },

            0xCB0F => {                                                             // RR A
                let r = self.rrc(self.reg.a);
                self.reg.a = r;
            },

            // RR r
            0xCB18 => {                                                             // RR B
                let r = self.rr(self.reg.b);
                self.reg.b = r;
            },

            0xCB19 => {                                                             // RR C
                let r = self.rr(self.reg.c);
                self.reg.c = r;
            },

            0xCB1A => {                                                             // RR D
                let r = self.rr(self.reg.d);
                self.reg.d = r;
            },

            0xCB1B => {                                                             // RR E
                let r = self.rr(self.reg.e);
                self.reg.e = r;
            },

            0xCB1C => {                                                             // RR H
                let r = self.rr(self.reg.h);
                self.reg.h = r;
            },

            0xCB1D => {                                                             // RR L
                let r = self.rr(self.reg.l);
                self.reg.l = r;
            },

            0xCB1E => {                                                             // RR (HL)
                let addr = self.reg.get_hl();
                let r = self.rr(self.bus.read_byte(addr));
                self.bus.write_byte(addr, r);
            },

            0xCB1F => {                                                             // RR A
                let r = self.rr(self.reg.a);
                self.reg.a = r;
            },

            // SLA r
            0xCB20 => {                                                             // SLA B
                let r = self.sla(self.reg.b);
                self.reg.b = r;
            },

            0xCB21 => {                                                             // SLA C
                let r = self.sla(self.reg.c);
                self.reg.c = r;
            },

            0xCB22 => {                                                             // SLA D
                let r = self.sla(self.reg.d);
                self.reg.d = r;
            },

            0xCB23 => {                                                             // SLA E
                let r = self.sla(self.reg.e);
                self.reg.e = r;
            },

            0xCB24 => {                                                             // SLA H
                let r = self.sla(self.reg.h);
                self.reg.h = r;
            },

            0xCB25 => {                                                             // SLA L
                let r = self.sla(self.reg.l);
                self.reg.l = r;
            },

            0xCB26 => {                                                             // SLA (HL)
                let addr = self.reg.get_hl();
                let r = self.sla(self.bus.read_byte(addr));
                self.bus.write_byte(addr, r);
            },

            0xCB27 => {                                                             // SLA A
                let r = self.sla(self.reg.a);
                self.reg.a = r;
            },

            // SRA r
            0xCB28 => {                                                             // SRA B
                let r = self.sra(self.reg.b);
                self.reg.b = r;
            },

            0xCB29 => {                                                             // SRA C
                let r = self.sra(self.reg.c);
                self.reg.c = r;
            },

            0xCB2A => {                                                             // SRA D
                let r = self.sra(self.reg.d);
                self.reg.d = r;
            },

            0xCB2B => {                                                             // SRA E
                let r = self.sra(self.reg.e);
                self.reg.e = r;
            },

            0xCB2C => {                                                             // SRA H
                let r = self.sra(self.reg.h);
                self.reg.h = r;
            },

            0xCB2D => {                                                             // SRA L
                let r = self.sra(self.reg.l);
                self.reg.l = r;
            },

            0xCB2E => {                                                             // SRA (HL)
                let addr = self.reg.get_hl();
                let r = self.sra(self.bus.read_byte(addr));
                self.bus.write_byte(addr, r);
            },

            0xCB2F => {                                                             // SRA A
                let r = self.sra(self.reg.a);
                self.reg.a = r;
            },

            // SRL r
            0xCB38 => {                                                             // SRL B
                let r = self.srl(self.reg.b);
                self.reg.b = r;
            },

            0xCB39 => {                                                             // SRL C
                let r = self.srl(self.reg.c);
                self.reg.c = r;
            },

            0xCB3A => {                                                             // SRL D
                let r = self.srl(self.reg.d);
                self.reg.d = r;
            },

            0xCB3B => {                                                             // SRL E
                let r = self.srl(self.reg.e);
                self.reg.e = r;
            },

            0xCB3C => {                                                             // SRL H
                let r = self.srl(self.reg.h);
                self.reg.h = r;
            },

            0xCB3D => {                                                             // SRL L
                let r = self.srl(self.reg.l);
                self.reg.l = r;
            },

            0xCB3E => {                                                             // SRL (HL)
                let addr = self.reg.get_hl();
                let r = self.srl(self.bus.read_byte(addr));
                self.bus.write_byte(addr, r);
            },

            0xCB3F => {                                                             // SRL A
                let r = self.srl(self.reg.a);
                self.reg.a = r;
            },

            // RLD
            0xED6F => {
                let hl_contents = self.bus.read_byte(self.reg.get_hl());
                let a_contents = self.reg.a;

                let r = (self.reg.a & 0xF0) | ((((hl_contents & 0xF0) as i8) >> 4) as u8);
                self.reg.a = r;
                self.bus.write_byte(self.reg.get_hl(), (hl_contents << 4) | (a_contents & 0x0F));
                self.reg.flags.s = (r as i8) < 0;
                self.reg.flags.z = r == 0x00;
                self.reg.flags.h = false;
                self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
                self.reg.flags.n = false;
            }

            // RRD
            0xED67 => {
                let hl_contents = self.bus.read_byte(self.reg.get_hl());
                let a_contents = self.reg.a;

                let r = (self.reg.a & 0xF0) | (hl_contents & 0x0F);
                self.reg.a = r;
                self.bus.write_byte(self.reg.get_hl(), ((a_contents & 0x0F) << 4 )| ((((hl_contents & 0xF0) as i8) >> 4) as u8));
                self.reg.flags.s = (r as i8) < 0;
                self.reg.flags.z = r == 0x00;
                self.reg.flags.h = false;
                self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
                self.reg.flags.n = false;
            }

            // Bit Set, Reset, and Test Group
            // BIT b,r
            0xCB40 ..= 0xCB7F => self.bit(self.bus.read_byte(self.reg.pc + 1)),

            // SET b,r
            0xCBC0 ..= 0xCBFF => self.set(self.bus.read_byte(self.reg.pc + 1)),

            // RES b,r
            0xCB80 ..= 0xCBBF => self.reset(self.bus.read_byte(self.reg.pc + 1)),

            // Jump group
            // JP (IX)
            0xDDE9 => { self.reg.pc = self.reg.get_ix(); },

            // JP (IY)
            0xFDE9 => { self.reg.pc = self.reg.get_iy(); },

            // Undocumented instructions
            // ADD A,IXH
            0xDD84 => {
                let n = self.reg.ixh;
                self.add(n);
            },

            // ADD A,IXL
            0xDD85 => {
                let n = self.reg.ixl;
                self.add(n);
            },

            // ADD A,IYH
            0xFD84 => {
                let n = self.reg.iyh;
                self.add(n);
            },

            // ADD A,IYL
            0xFD85 => {
                let n = self.reg.iyl;
                self.add(n);
            },

            // ADC A,IXH
            0xDD8C => {
                let n = self.reg.ixh;
                self.adc(n);
            },

            // ADC A,IXL
            0xDD8D => {
                let n = self.reg.ixl;
                self.adc(n);
            },

            // ADC A,IYH
            0xFD8C => {
                let n = self.reg.iyh;
                self.adc(n);
            },

            // ADC A,IYL
            0xFD8D => {
                let n = self.reg.iyl;
                self.adc(n);
            },

            // SUB IXH
            0xDD94 => {
                let n = self.reg.ixh;
                self.sub(n);
            },

            // SUB IXL
            0xDD95 => {
                let n = self.reg.ixl;
                self.sub(n);
            },

            // SUB IYH
            0xFD94 => {
                let n = self.reg.iyh;
                self.sub(n);
            },

            // SUB IYL
            0xFD95 => {
                let n = self.reg.iyl;
                self.sub(n);
            },

            // SBC A,IXH
            0xDD9C => {
                let n = self.reg.ixh;
                self.sbc(n);
            },

            // SBC A,IXL
            0xDD9D => {
                let n = self.reg.ixl;
                self.sbc(n);
            },

            // SBC A,IYH
            0xFD9C => {
                let n = self.reg.iyh;
                self.sbc(n);
            },

            // SBC A,IYL
            0xFD9D => {
                let n = self.reg.iyl;
                self.sbc(n);
            },

            // AND IXH
            0xDDA4 => {
                let n = self.reg.ixh;
                self.and(n);
            },

            // AND IXL
            0xDDA5 => {
                let n = self.reg.ixl;
                self.and(n);
            },

            // AND IYH
            0xFDA4 => {
                let n = self.reg.iyh;
                self.and(n);
            },

            // AND IYL
            0xFDA5 => {
                let n = self.reg.iyl;
                self.and(n);
            },

            // OR IXH
            0xDDB4 => {
                let n = self.reg.ixh;
                self.or(n);
            },

            // OR IXL
            0xDDB5 => {
                let n = self.reg.ixl;
                self.or(n);
            },

            // OR IYH
            0xFDB4 => {
                let n = self.reg.iyh;
                self.or(n);
            },

            // OR IYL
            0xFDB5 => {
                let n = self.reg.iyl;
                self.or(n);
            },

            // XOR IXH
            0xDDAC => {
                let n = self.reg.ixh;
                self.xor(n);
            },

            // XOR IXL
            0xDDAD => {
                let n = self.reg.ixl;
                self.xor(n);
            },

            // XOR IYH
            0xFDAC => {
                let n = self.reg.iyh;
                self.xor(n);
            },

            // XOR IYL
            0xFDAD => {
                let n = self.reg.iyl;
                self.xor(n);
            },

            // CP IXH
            0xDDBC => {
                let n = self.reg.ixh;
                self.cp(n);
            },

            // CP IXL
            0xDDBD => {
                let n = self.reg.ixl;
                self.cp(n);
            },

            // CP IYH
            0xFDBC => {
                let n = self.reg.iyh;
                self.cp(n);
            },

            // CP IYL
            0xFDBD => {
                let n = self.reg.iyl;
                self.cp(n);
            },

            // INC IXH
            0xDD24 => {
                let n = self.reg.ixh;
                let r = self.inc(n);
                self.reg.ixh = r;
            },

            // DEC IXH
            0xDD25 => {
                let n = self.reg.ixh;
                let r = self.dec(n);
                self.reg.ixh = r;
            },

            // INC IXL
            0xDD2C => {
                let n = self.reg.ixl;
                let r = self.inc(n);
                self.reg.ixl = r;
            },

            // DEC IXL
            0xDD2D => {
                let n = self.reg.ixl;
                let r = self.dec(n);
                self.reg.ixl = r;
            },

            // LD IXH,n
            0xDD26 => {
                let n = self.bus.read_byte(self.reg.pc + 3);
                self.reg.ixh = n;
            }

            // LD IYH,n
            0xFD26 => {
                let n = self.bus.read_byte(self.reg.pc + 3);
                self.reg.iyh = n;
            }

            // LD IXL,n
            0xDD2E => {
                let n = self.bus.read_byte(self.reg.pc + 3);
                self.reg.ixl = n;
            },

            // LD IYL,n
            0xFD2E => {
                let n = self.bus.read_byte(self.reg.pc + 3);
                self.reg.iyl = n;
            },

            // LD B,B
            0xDD40 | 0xFD40 => {},

            // LD B,C
            0xDD41 | 0xFD41 => self.reg.b = self.reg.c,

            // LD B,D
            0xDD42 | 0xFD42 => self.reg.b = self.reg.d,

            // LD B,E
            0xDD43 | 0xFD43 => self.reg.b = self.reg.e,

            // LD B,IXH
            0xDD44 => self.reg.b = self.reg.ixh,

            // LD B,IYH
            0xFD44 => self.reg.b = self.reg.iyh,

            // LD B,IXL
            0xDD45 => self.reg.b = self.reg.ixl,

            // LD B,IYL
            0xFD45 => self.reg.b = self.reg.iyl,

            // LD B,A
            0xDD47 | 0xFD47 => self.reg.b = self.reg.a,

            // LD C,B
            0xDD48 | 0xFD48 => self.reg.c = self.reg.b,

            // LD C,C
            0xDD49 | 0xFD49 => {},

            // LD C,D
            0xDD4A | 0xFD4A => self.reg.c = self.reg.d,

            // LD C,E
            0xDD4B | 0xFD4B => self.reg.c = self.reg.e,

            // LD C,IXH
            0xDD4C => self.reg.c = self.reg.ixh,

            // LD C,IYH
            0xFD4C => self.reg.c = self.reg.iyh,

            // LD C,IXL
            0xDD4D => self.reg.c = self.reg.ixl,

            // LD C,IYL
            0xFD4D => self.reg.c = self.reg.iyl,

            // LD C,A
            0xDD4F | 0xFD4F => self.reg.c = self.reg.a,

            // LD D,B
            0xDD50 | 0xFD50 => self.reg.d = self.reg.b,

            // LD D,C
            0xDD51 | 0xFD51 => self.reg.d = self.reg.c,

            // LD D,D
            0xDD52 | 0xFD52 => {},

            // LD D,E
            0xDD53 | 0xFD53 => self.reg.d = self.reg.e,

            // LD D,IXH
            0xDD54 => self.reg.d = self.reg.ixh,

            // LD D,IYH
            0xFD54 => self.reg.d = self.reg.iyh,

            // LD D,IXL
            0xDD55 => self.reg.d = self.reg.ixl,

            // LD D,IYL
            0xFD55 => self.reg.d = self.reg.iyl,

            // LD D,A
            0xDD57 | 0xFD57 => self.reg.d = self.reg.a,

            // LD E,B
            0xDD58 | 0xFD58 => self.reg.e = self.reg.b,

            // LD E,C
            0xDD59 | 0xFD59 => self.reg.e = self.reg.c,

            // LD E,D
            0xDD5A | 0xFD5A => self.reg.e = self.reg.d,

            // LD E,E
            0xDD5B | 0xFD5B => {},

            // LD E,IXH
            0xDD5C => self.reg.e = self.reg.ixh,

            // LD E,IYH
            0xFD5C => self.reg.e = self.reg.iyh,

            // LD E,IXL
            0xDD5D => self.reg.e = self.reg.ixl,

            // LD E,IYL
            0xFD5D => self.reg.e = self.reg.iyl,

            // LD E,A
            0xDD5F | 0xFD5F => self.reg.e = self.reg.a,

            // LD IXH,B
            0xDD60 => self.reg.ixh = self.reg.b,

            // LD IYH,B
            0xFD60 => self.reg.iyh = self.reg.b,

            // LD IXH,C
            0xDD61 => self.reg.ixh = self.reg.c,

            // LD IYH,C
            0xFD61 => self.reg.iyh = self.reg.c,

            // LD IXH,D
            0xDD62 => self.reg.ixh = self.reg.d,

            // LD IYH,D
            0xFD62 => self.reg.iyh = self.reg.d,

            // LD IXH,E
            0xDD63 => self.reg.ixh = self.reg.e,

            // LD IYH,E
            0xFD63 => self.reg.iyh = self.reg.e,

            // LD IXH,IXH
            0xDD64 => {},

            // LD IYH,IYH
            0xFD64 => {},

            // LD IXH,IXL
            0xDD65 => self.reg.ixh = self.reg.ixl,

            // LD IYH,IYL
            0xFD65 => self.reg.iyh = self.reg.iyl,

            // LD IXH,A
            0xDD67 => self.reg.ixh = self.reg.a,

            // LD IYH,A
            0xFD67 => self.reg.iyh = self.reg.a,

            // LD IXL,B
            0xDD68 => self.reg.ixl = self.reg.b,

            // LD IYL,B
            0xFD68 => self.reg.iyl = self.reg.b,

            // LD IXL,C
            0xDD69 => self.reg.ixl = self.reg.c,

            // LD IYL,C
            0xFD69 => self.reg.iyl = self.reg.c,

            // LD IXL,D
            0xDD6A => self.reg.ixl = self.reg.d,

            // LD IYL,D
            0xFD6A => self.reg.iyl = self.reg.d,

            // LD IXL,E
            0xDD6B => self.reg.ixl = self.reg.e,

            // LD IYL,E
            0xFD6B => self.reg.iyl = self.reg.e,

            // LD IXL,IXH
            0xDD6C => self.reg.ixl = self.reg.ixh,

            // LD IYL,IYH
            0xFD6C => self.reg.iyl = self.reg.iyh,

            // LD IXL,IXL
            0xDD6D => {},

            // LD IYL,IYL
            0xFD6D => {},

            // LD IXL,A
            0xDD6F => self.reg.ixl = self.reg.a,

            // LD IYL,A
            0xFD6F => self.reg.iyl = self.reg.a,

            // LD A,B
            0xDD78 | 0xFD78 => self.reg.a = self.reg.b,

            // LD A,C
            0xDD79 | 0xFD79 => self.reg.a = self.reg.c,

            // LD A,D
            0xDD7A | 0xFD7A => self.reg.a = self.reg.d,

            // LD A,E
            0xDD7B | 0xFD7B => self.reg.a = self.reg.e,

            // LD A,IXH
            0xDD7C => self.reg.a = self.reg.ixh,

            // LD A,IYH
            0xFD7C => self.reg.a = self.reg.iyh,

            // LD A,IXL
            0xDD7D => self.reg.a = self.reg.ixl,

            // LD A,IYL
            0xFD7D => self.reg.a = self.reg.iyl,

            // LD A,A
            0xDD7F | 0xFD7F => {},

            // SLL B
            0xCB30 => {
                let r = self.sll(self.reg.b);
                self.reg.b = r;
            },

            // SLL C
            0xCB31 => {
                let r = self.sll(self.reg.c);
                self.reg.c = r;
            },

            // SLL D
            0xCB32 => {
                let r = self.sll(self.reg.d);
                self.reg.d = r;
            },

            // SLL E
            0xCB33 => {
                let r = self.sll(self.reg.e);
                self.reg.e = r;
            },

            // SLL H
            0xCB34 => {
                let r = self.sll(self.reg.h);
                self.reg.h = r;
            },

            // SLL L
            0xCB35 => {
                let r = self.sll(self.reg.l);
                self.reg.l = r;
            },

            // SLL (HL)
            0xCB36 => {
                let addr = self.reg.get_hl();
                let r = self.sll(self.bus.read_byte(addr));
                self.bus.write_byte(addr, r);
            },

            // SLL A
            0xCB37 => {
                let r = self.sll(self.reg.a);
                self.reg.a = r;
            },

            // Input and Output Group
            // IN B,(C)
            0xED40 => self.reg.b = self.inrc(),
            
            // IN C,(C)
            0xED48 => self.reg.c = self.inrc(),

            // IN D,(C)
            0xED50 => self.reg.d = self.inrc(),

            // IN E,(C)
            0xED58 => self.reg.e = self.inrc(),

            // IN H,(C)
            0xED60 => self.reg.h = self.inrc(),

            // IN B,(C)
            0xED68 => self.reg.l = self.inrc(),

            _ => {
                if self.debug.unknw_instr { self.debug.string = format!("{:#06X}", opcode); }
                cycles = 0xFF;
            }
        }

        match opcode {
            0xDDE9 | 0xFDE9 | 0xED4D | 0xED45 => {},
            0xDD46 | 0xFD46 | 0xDD4E | 0xFD4E | 0xDD56 | 0xFD56 |
            0xDD5E | 0xFD5E | 0xDD66 | 0xFD66 | 0xDD6E | 0xFD6E |
            0xDD7E | 0xFD7E |
            0xDD70 | 0xDD71 | 0xDD72 | 0xDD73 | 0xDD74 | 0xDD75 |
            0xDD77 |
            0xFD70 | 0xFD71 | 0xFD72 | 0xFD73 | 0xFD74 | 0xFD75 |
            0xFD77 | 0xDD86 | 0xFD86 | 0xDD8E | 0xFD8E |
            0xDD96 | 0xFD96 | 0xDD9E | 0xFD9E | 0xDDA6 | 0xFDA6 |
            0xDDB6 | 0xFDB6 | 0xDDAE | 0xFDAE | 0xDDBE | 0xFDBE |
            0xDD34 | 0xFD34 | 0xDD35 | 0xFD35=> self.reg.pc += 3,
            0xDD36 | 0xFD36 | 0xDD21 | 0xFD21 | 0xED4B | 0xED5B |
            0xED6B | 0xED7B | 0xDD2A | 0xFD2A |
            0xED43 | 0xED53 | 0xED63 | 0xED73 |
            0xDD22 | 0xFD22 | 0xDDCB | 0xFDCB => self.reg.pc += 4,
            _ => self.reg.pc +=2,
        }

        if self.debug.opcode == true { self.debug.string = format!("{:#06X}", opcode) }

        cycles
    }

    fn execute_1byte(&mut self, opcode: u8) -> u32 {
        let mut cycles = CYCLES[opcode as usize].into();

        match opcode {
            // 8-Bit Load Group
            // LD r,r'      LD r,(HL)
            0x40 => {},                                                 // LD B,B
            0x41 => self.reg.b = self.reg.c,                            // LD B,C
            0x42 => self.reg.b = self.reg.d,                            // LD B,D
            0x43 => self.reg.b = self.reg.e,                            // LD B,E
            0x44 => self.reg.b = self.reg.h,                            // LD B,H
            0x45 => self.reg.b = self.reg.l,                            // LD B,L
            0x46 => {                                                   // LD B,(HL)
                let addr = self.reg.get_hl();
                self.reg.b = self.bus.read_byte(addr)
            }
            0x47 => self.reg.b = self.reg.a,                            // LD B,A

            0x48 => self.reg.c = self.reg.b,                            // LD C,B
            0x49 => {},                                                 // LD C,C
            0x4A => self.reg.c = self.reg.d,                            // LD C,D
            0x4B => self.reg.c = self.reg.e,                            // LD C,E
            0x4C => self.reg.c = self.reg.h,                            // LD C,H
            0x4D => self.reg.c = self.reg.l,                            // LD C,L
            0x4E => {                                                   // LD C,(HL)
                let addr = self.reg.get_hl();
                self.reg.c = self.bus.read_byte(addr)
            }
            0x4F => self.reg.c = self.reg.a,                            // LD C,A

            0x50 => self.reg.d = self.reg.b,                            // LD D,B
            0x51 => self.reg.d = self.reg.c,                            // LD D,C
            0x52 => {},                                                 // LD D,D
            0x53 => self.reg.d = self.reg.e,                            // LD D,E
            0x54 => self.reg.d = self.reg.h,                            // LD D,H
            0x55 => self.reg.d = self.reg.l,                            // LD D,L
            0x56 => {                                                   // LD D,(HL)
                let addr = self.reg.get_hl();
                self.reg.d = self.bus.read_byte(addr)
            }
            0x57 => self.reg.d = self.reg.a,                            // LD D,A

            0x58 => self.reg.e = self.reg.b,                            // LD E,B
            0x59 => self.reg.e = self.reg.c,                            // LD E,C
            0x5A => self.reg.e = self.reg.d,                            // LD E,D
            0x5B => {},                                                 // LD E,E
            0x5C => self.reg.e = self.reg.h,                            // LD E,H
            0x5D => self.reg.e = self.reg.l,                            // LD E,L
            0x5E => {                                                   // LD E,(HL)
                let addr = self.reg.get_hl();
                self.reg.e = self.bus.read_byte(addr)
            }
            0x5F => self.reg.e = self.reg.a,                            // LD E,A

            0x60 => self.reg.h = self.reg.b,                            // LD H,B
            0x61 => self.reg.h = self.reg.c,                            // LD H,C
            0x62 => self.reg.h = self.reg.d,                            // LD H,D
            0x63 => self.reg.h = self.reg.e,                            // LD H,E
            0x64 => {},                                                 // LD H,H
            0x65 => self.reg.h = self.reg.l,                            // LD H,L
            0x66 => {                                                   // LD H,(HL)
                let addr = self.reg.get_hl();
                self.reg.h = self.bus.read_byte(addr)
            }
            0x67 => self.reg.h = self.reg.a,                            // LD H,A

            0x68 => self.reg.l = self.reg.b,                            // LD L,B
            0x69 => self.reg.l = self.reg.c,                            // LD L,C
            0x6A => self.reg.l = self.reg.d,                            // LD L,D
            0x6B => self.reg.l = self.reg.e,                            // LD L,E
            0x6C => self.reg.l = self.reg.h,                            // LD L,H
            0x6D => {},                                                 // LD L,L
            0x6E => {                                                   // LD L,(HL)
                let addr = self.reg.get_hl();
                self.reg.l = self.bus.read_byte(addr)
            }
            0x6F => self.reg.l = self.reg.a,                            // LD L,A

            0x78 => self.reg.a = self.reg.b,                            // LD A,B
            0x79 => self.reg.a = self.reg.c,                            // LD A,C
            0x7A => self.reg.a = self.reg.d,                            // LD A,D
            0x7B => self.reg.a = self.reg.e,                            // LD A,E
            0x7C => self.reg.a = self.reg.h,                            // LD A,H
            0x7D => self.reg.a = self.reg.l,                            // LD A,L
            0x7E => {                                                   // LD A,(HL)
                let addr = self.reg.get_hl();
                self.reg.a = self.bus.read_byte(addr)
            }
            0x7F => {},                                                             // LD A,A

            // LD (HL),r
            0x70 => {                                                               // LD (HL), B
                let addr = self.reg.get_hl();
                self.bus.write_byte(addr, self.reg.b)
            },
            0x71 => {                                                               // LD (HL), C
                let addr = self.reg.get_hl();
                self.bus.write_byte(addr, self.reg.c)
            },
            0x72 => {                                                               // LD (HL), D
                let addr = self.reg.get_hl();
                self.bus.write_byte(addr, self.reg.d)
            },
            0x73 => {                                                               // LD (HL), E
                let addr = self.reg.get_hl();
                self.bus.write_byte(addr, self.reg.e)
            },
            0x74 => {                                                               // LD (HL), H
                let addr = self.reg.get_hl();
                self.bus.write_byte(addr, self.reg.h)
            },
            0x75 => {                                                               // LD (HL), L
                let addr = self.reg.get_hl();
                self.bus.write_byte(addr, self.reg.l)
            },

            0x77 => {                                                               // LD (HL), A
                let addr = self.reg.get_hl();
                self.bus.write_byte(addr, self.reg.a)
            },

            // LD r,n
            0x06 => {                                                               // LD B,n
                let data = self.bus.read_byte(self.reg.pc + 1);
                self.reg.b = data;
            },
            0x0E => {                                                               // LD C,n
                let data = self.bus.read_byte(self.reg.pc + 1);
                self.reg.c = data;
            },
            0x16 => {                                                               // LD D,n
                let data = self.bus.read_byte(self.reg.pc + 1);
                self.reg.d = data;
            },
            0x1E => {                                                               // LD E,n
                let data = self.bus.read_byte(self.reg.pc + 1);
                self.reg.e = data;
            },
            0x26 => {                                                               // LD H,n
                let data = self.bus.read_byte(self.reg.pc + 1);
                self.reg.h = data;
            },
            0x2E => {                                                               // LD L,n
                let data = self.bus.read_byte(self.reg.pc + 1);
                self.reg.l = data;
            },
            0x36 => {                                                               // LD (HL),n
                let data = self.bus.read_byte(self.reg.pc + 1);
                let addr = self.reg.get_hl();
                self.bus.write_byte(addr, data);
            },
            0x3E => {                                                               // LD A,n
                let data = self.bus.read_byte(self.reg.pc + 1);
                self.reg.a = data;
            },

            // LD A,(BC)
            0x0A => {
                let addr = self.reg.get_bc();
                self.reg.a = self.bus.read_byte(addr);
            },

            // LD A,(DE)
            0x1A => {
                let addr = self.reg.get_de();
                self.reg.a = self.bus.read_byte(addr);
            },

            // LD A,(nn)
            0x3A => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                self.reg.a = self.bus.read_byte(addr);
            },

            // LD (BC),A
            0x02 => {
                let addr = self.reg.get_bc();
                self.bus.write_byte(addr, self.reg.a);
            },

            // LD (DE),A
            0x12 => {
                let addr = self.reg.get_de();
                self.bus.write_byte(addr, self.reg.a);
            },

            // LD (nn),A
            0x32 => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                self.bus.write_byte(addr, self.reg.a);
            },

            // 16-Bit Load Group
            // LD dd,nn
            0x01 => {                                                               // LD BC,nn
                let d16 = self.bus.read_word(self.reg.pc + 1); 
                self.reg.set_bc(d16);
            },
            0x11 => {                                                               // LD DE,nn
                let d16 = self.bus.read_word(self.reg.pc + 1); 
                self.reg.set_de(d16);
            },
            0x21 => {                                                               // LD HL,nn
                let d16 = self.bus.read_word(self.reg.pc + 1); 
                self.reg.set_hl(d16);
            },
            0x31 => {                                                               // LD SP,nn
                let d16 = self.bus.read_word(self.reg.pc + 1); 
                self.reg.sp = d16;
            },

            // LD HL,(nn)
            0x2A => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                let d = self.bus.read_word(addr);
                self.reg.set_hl(d);
            },

            // LD (nn),HL
            0x22 => {
                let d = self.reg.get_hl();
                let addr = self.bus.read_word(self.reg.pc + 1);
                self.bus.write_word(addr, d);
            },

            // LD SP,HL
            0xF9 => self.reg.sp = self.reg.get_hl(),

            // PUSH qq
            0xC5 => {                                                               // PUSH BC
                self.reg.sp = self.reg.sp.wrapping_sub(2);
                self.bus.write_word(self.reg.sp, self.reg.get_bc());
            },
            0xD5 => {                                                               // PUSH DE
                self.reg.sp = self.reg.sp.wrapping_sub(2);
                self.bus.write_word(self.reg.sp, self.reg.get_de());
            },
            0xE5 => {                                                               // PUSH HL
                self.reg.sp = self.reg.sp.wrapping_sub(2);
                self.bus.write_word(self.reg.sp, self.reg.get_hl());
            },
            0xF5 => {                                                               // PUSH AF
                self.reg.sp = self.reg.sp.wrapping_sub(2);
                self.bus.write_byte(self.reg.sp, self.reg.flags.to_byte());
                self.bus.write_byte(self.reg.sp + 1, self.reg.a);
            },

            // POP qq
            0xC1 => {                                                               // POP BC
                self.reg.set_bc(self.bus.read_word(self.reg.sp));
                self.reg.sp = self.reg.sp.wrapping_add(2);
            },

            0xD1 => {                                                               // POP DE
                self.reg.set_de(self.bus.read_word(self.reg.sp));
                self.reg.sp = self.reg.sp.wrapping_add(2);
            },

            0xE1 => {                                                               // POP HL
                self.reg.set_hl(self.bus.read_word(self.reg.sp));
                self.reg.sp = self.reg.sp.wrapping_add(2);
            },

            0xF1 => {                                                               // POP AF
                self.reg.a = self.bus.read_byte((self.reg.sp)+1);
                let bflags = self.bus.read_byte(self.reg.sp);
                self.reg.flags.from_byte(bflags);
                self.reg.sp = self.reg.sp.wrapping_add(2);
            },

            // Exchange, Block Transfer, and Search Group
            // EX DE,HL
            0xEB => {
                let de= self.reg.get_de();
                let hl = self.reg.get_hl();
                self.reg.set_de(hl);
                self.reg.set_hl(de);
            }

            // EX AF,AF'
            0x08 => {
                let af = self.reg.get_af();
                let afp = self.alt.get_af();
                self.reg.set_af(afp);
                self.alt.set_af(af);
            }

            // EXX
            0xD9 => {
                let bc = self.reg.get_bc();
                let de = self.reg.get_de();
                let hl = self.reg.get_hl();
                let bcp = self.alt.get_bc();
                let dep = self.alt.get_de();
                let hlp = self.alt.get_hl();
                self.reg.set_bc(bcp);
                self.reg.set_de(dep);
                self.reg.set_hl(hlp);
                self.alt.set_bc(bc);
                self.alt.set_de(de);
                self.alt.set_hl(hl);
            }

            // EX (SP),HL
            0xE3 => {
                let pointed_by_sp = self.bus.read_word(self.reg.sp);
                let hl = self.reg.get_hl();
                self.bus.write_word(self.reg.sp, hl);
                self.reg.set_hl(pointed_by_sp);
            },

            // 8-Bit Arithmetic Group
            // ADD A,r
            0x80 => self.add(self.reg.b),                                   // ADD A,B
            0x81 => self.add(self.reg.c),                                   // ADD A,C
            0x82 => self.add(self.reg.d),                                   // ADD A,D
            0x83 => self.add(self.reg.e),                                   // ADD A,E
            0x84 => self.add(self.reg.h),                                   // ADD A,H
            0x85 => self.add(self.reg.l),                                   // ADD A,L
            0x86 => {                                                       // ADD (HL)
                let addr = self.reg.get_hl();
                let n = self.bus.read_byte(addr);
                self.add(n)
            },
            0x87 => self.add(self.reg.a),                                    // ADD A,A

            // ADD A,n
            0xC6 => {
                let n = self.bus.read_byte(self.reg.pc + 1);
                self.add(n);
            },

            // ADC A,r
            0x88 => self.adc(self.reg.b),                                    // ADC A,B
            0x89 => self.adc(self.reg.c),                                    // ADC A,C
            0x8A => self.adc(self.reg.d),                                    // ADC A,D
            0x8B => self.adc(self.reg.e),                                    // ADC A,E
            0x8C => self.adc(self.reg.h),                                    // ADC A,H
            0x8D => self.adc(self.reg.l),                                    // ADC A,L
            0x8E => {                                                        // ADC A,(HL)
                let addr = self.reg.get_hl();
                let n = self.bus.read_byte(addr);
                self.adc(n)
            },
            0x8F => self.adc(self.reg.a),                                    // ADC A,A

            // ADC a,n
            0xCE => {                                                        // ADC A,(HL)
                let n = self.bus.read_byte(self.reg.pc + 1);
                self.adc(n)
            },

            // SUB s
            0x90 => self.sub(self.reg.b),                                     // SUB B
            0x91 => self.sub(self.reg.c),                                     // SUB C
            0x92 => self.sub(self.reg.d),                                     // SUB D
            0x93 => self.sub(self.reg.e),                                     // SUB E
            0x94 => self.sub(self.reg.h),                                     // SUB H
            0x95 => self.sub(self.reg.l),                                     // SUB L
            0x96 => {                                                         // SUB (HL)
                let addr = self.reg.get_hl();
                let n = self.bus.read_byte(addr);
                self.sub(n)
            },
            0x97 => self.sub(self.reg.a),                                     // SUB A

            0xD6 => {                                                         // SUB n
                let n = self.bus.read_byte(self.reg.pc + 1);
                self.sub(n);
            },

            // SBC A,s
            0x98 => self.sbc(self.reg.b),                                    // SBC A,B
            0x99 => self.sbc(self.reg.c),                                    // SBC A,C
            0x9A => self.sbc(self.reg.d),                                    // SBC A,D
            0x9B => self.sbc(self.reg.e),                                    // SBC A,E
            0x9C => self.sbc(self.reg.h),                                    // SBC A,H
            0x9D => self.sbc(self.reg.l),                                    // SBC A,L
            0x9E => {                                                        // SBC A,(HL)
                let addr = self.reg.get_hl();
                let n = self.bus.read_byte(addr);
                self.sbc(n)
            },
            0x9F => self.sbc(self.reg.a),                                    // SBC A,A

            0xDE => {                                                        // SBC A,n
                let n = self.bus.read_byte(self.reg.pc + 1);
                self.sbc(n);
            },

            // AND s
            0xA0 => self.and(self.reg.b),                                     // AND B
            0xA1 => self.and(self.reg.c),                                     // AND C
            0xA2 => self.and(self.reg.d),                                     // AND D
            0xA3 => self.and(self.reg.e),                                     // AND E
            0xA4 => self.and(self.reg.h),                                     // AND H
            0xA5 => self.and(self.reg.l),                                     // AND L
            0xA6 => {                                                         // AND (HL)
                let addr = self.reg.get_hl();
                let n = self.bus.read_byte(addr);
                self.and(n)
            },
            0xA7 => self.and(self.reg.a),                                     // AND A

            0xE6 => {                                                         // AND n
                let n = self.bus.read_byte(self.reg.pc + 1);
                self.and(n);
            },

            // OR s
            0xB0 => self.or(self.reg.b),                                      // OR B
            0xB1 => self.or(self.reg.c),                                      // OR C
            0xB2 => self.or(self.reg.d),                                      // OR D
            0xB3 => self.or(self.reg.e),                                      // OR E
            0xB4 => self.or(self.reg.h),                                      // OR H
            0xB5 => self.or(self.reg.l),                                      // OR L
            0xB6 => {                                                         // OR (HL)
                let addr = self.reg.get_hl();
                let n = self.bus.read_byte(addr);
                self.or(n)
            },
            0xB7 => self.or(self.reg.a),                                      // OR A

            0xF6 => {                                                         // OR n
                let n = self.bus.read_byte(self.reg.pc + 1);
                self.or(n);
            },

            // XOR s
            0xA8 => self.xor(self.reg.b),                                     // XOR B
            0xA9 => self.xor(self.reg.c),                                     // XOR C
            0xAA => self.xor(self.reg.d),                                     // XOR D
            0xAB => self.xor(self.reg.e),                                     // XOR E
            0xAC => self.xor(self.reg.h),                                     // XOR H
            0xAD => self.xor(self.reg.l),                                     // XOR L
            0xAE => {                                                         // XOR (HL)
                let addr = self.reg.get_hl();
                let n = self.bus.read_byte(addr);
                self.xor(n)
            },
            0xAF => self.xor(self.reg.a),                                     // XOR A

            0xEE => {                                                         // XOR n
                let n = self.bus.read_byte(self.reg.pc + 1);
                self.xor(n);
            },

            // CMP s
            0xB8 => self.cp(self.reg.b),                                      // CP B
            0xB9 => self.cp(self.reg.c),                                      // CP C
            0xBA => self.cp(self.reg.d),                                      // CP D
            0xBB => self.cp(self.reg.e),                                      // CP E
            0xBC => self.cp(self.reg.h),                                      // CP H
            0xBD => self.cp(self.reg.l),                                      // CP L
            0xBE => {                                                         // CP (HL)
                let addr = self.reg.get_hl();
                let n = self.bus.read_byte(addr);
                self.cp(n)
            },
            0xBF => self.cp(self.reg.a),                                      // CP A

            0xFE => {                                                         // CP n
                let n = self.bus.read_byte(self.reg.pc + 1);
                self.cp(n);
            },

            // INC r
            0x04 => self.reg.b = self.inc(self.reg.b),                  // INC B
            0x0C => self.reg.c = self.inc(self.reg.c),                  // INC C
            0x14 => self.reg.d = self.inc(self.reg.d),                  // INC D
            0x1C => self.reg.e = self.inc(self.reg.e),                  // INC E
            0x24 => self.reg.h = self.inc(self.reg.h),                  // INC H
            0x2C => self.reg.l = self.inc(self.reg.l),                  // INC L
            0x34 => {                                                   // INC (HL)
                let addr = self.reg.get_hl();
                let r = self.inc(self.bus.read_byte(addr));
                self.bus.write_byte(addr, r);
            },
            0x3C => self.reg.a = self.inc(self.reg.a),                  // INC A

            // DEC m
            0x05 => self.reg.b = self.dec(self.reg.b),                  // DEC B
            0x0D => self.reg.c = self.dec(self.reg.c),                  // DEC C
            0x15 => self.reg.d = self.dec(self.reg.d),                  // DEC D
            0x1D => self.reg.e = self.dec(self.reg.e),                  // DEC E
            0x25 => self.reg.h = self.dec(self.reg.h),                  // DEC H
            0x2D => self.reg.l = self.dec(self.reg.l),                  // DEC L
            0x35 => {                                                   // DEC (HL)
                let addr = self.reg.get_hl();
                let r = self.dec(self.bus.read_byte(addr));
                self.bus.write_byte(addr, r);
            },
            0x3D => self.reg.a = self.dec(self.reg.a),                  // DEC A

            // General-Purpose Arithmetic and CPU Control Groups
            // DAA
            0x27 => self.daa(),

            // CPL
            0x2F => {
                self.reg.a = !self.reg.a;
                self.reg.flags.h = true;
                self.reg.flags.n = true;
            },

            // CCF
            0x3F => {
                self.reg.flags.h = self.reg.flags.c;
                self.reg.flags.c = !self.reg.flags.c;
                self.reg.flags.n = false;
            },

            // SCF
            0x37 => {
                self.reg.flags.c = true;
                self.reg.flags.h = false;
                self.reg.flags.n = false;
            },

            // NOP
            0x00 => {},

            // HALT
            0x76 => self.halt = true,

            // DI
            0xF3 => {
                self.iff1 = false;
                self.iff2 = false;
            },

            // EI
            0xFB => {
                self.iff1 = true;
                self.iff2 = true;
            },

            // 16-Bit Arithmetic Group
            // ADD HL,ss
            0x09 => {                                                       // ADD HL,BC
                let reg = self.reg.get_bc();
                let r = self.add_16(self.reg.get_hl(), reg);
                self.reg.set_hl(r);
            },
            0x19 => {                                                       // ADD HL,DE
                let reg = self.reg.get_de();
                let r = self.add_16(self.reg.get_hl(), reg);
                self.reg.set_hl(r);
            },
            0x29 => {                                                       // ADD HL,HL
                let reg = self.reg.get_hl();
                let r = self.add_16(self.reg.get_hl(), reg);
                self.reg.set_hl(r);
            },
            0x39 => {                                                       // ADD HL,SP
                let reg = self.reg.sp;
                let r = self.add_16(self.reg.get_hl(), reg);
                self.reg.set_hl(r);
            },

            // INC ss
            0x03 => {                                                       // INC BC
                let r = self.reg.get_bc().wrapping_add(1);
                self.reg.set_bc(r);
            },

            0x13 => {                                                       // INC DE
                let r = self.reg.get_de().wrapping_add(1);
                self.reg.set_de(r);
            },

            0x23 => {                                                       // INC HL
                let r = self.reg.get_hl().wrapping_add(1);
                self.reg.set_hl(r);
            },

            0x33 => {                                                       // INC SP
                let r = self.reg.sp.wrapping_add(1);
                self.reg.sp = r;
            },

            // DEC ss
            0x0B => {                                                       // DEC BC
                let r = self.reg.get_bc().wrapping_sub(1);
                self.reg.set_bc(r);
            },

            0x1B => {                                                       // DEC DE
                let r = self.reg.get_de().wrapping_sub(1);
                self.reg.set_de(r);
            },

            0x2B => {                                                       // DEC HL
                let r = self.reg.get_hl().wrapping_sub(1);
                self.reg.set_hl(r);
            },

            0x3B => {                                                       // DEC SP
                let r = self.reg.sp.wrapping_sub(1);
                self.reg.sp = r;
            },

            // Rotate and Shift Group
            // RLCA
            0x07 => self.rlca(),

            // RLA
            0x17 => self.rla(),

            // RRCA
            0x0F => self.rrca(),

            // RRA
            0x1F => self.rra(),

            // Jump group
            // JP nn
            0xC3 => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                self.reg.pc = addr;
            },

            // JP C,nn
            0xDA => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if self.reg.flags.c { self.reg.pc = addr; } else { self.reg.pc += 3 }
            },

            // JP NC,nn
            0xD2 => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if !self.reg.flags.c { self.reg.pc = addr; } else { self.reg.pc += 3 }
            },

            // JP Z,nn
            0xCA => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if self.reg.flags.z { self.reg.pc = addr; } else { self.reg.pc += 3 }
            },

            // JP NZ,nn
            0xC2 => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if !self.reg.flags.z { self.reg.pc = addr; } else { self.reg.pc += 3 }
            },

            // JP M,nn
            0xFA => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if self.reg.flags.s { self.reg.pc = addr; } else { self.reg.pc += 3 }
            },

            // JP P,nn
            0xF2 => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if !self.reg.flags.s { self.reg.pc = addr; } else { self.reg.pc += 3 }
            },

            // JP PE,nn
            0xEA => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if self.reg.flags.p { self.reg.pc = addr; } else { self.reg.pc += 3 }
            },

            // JP PO,nn
            0xE2 => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if !self.reg.flags.p { self.reg.pc = addr; } else { self.reg.pc += 3 }
            },

            // JR e
            0x18 => {
                let displacement= self.bus.read_byte(self.reg.pc + 1);
                if bit::get(displacement, 7) { self.reg.pc = self.reg.pc - ( signed_to_abs(displacement) as u16 ) }
                else { self.reg.pc = self.reg.pc + ( displacement as u16 ) }
            },

            // JR C,e
            0x38 => {
                if self.reg.flags.c {
                    let displacement= self.bus.read_byte(self.reg.pc + 1);
                    if bit::get(displacement, 7) { self.reg.pc = self.reg.pc - ( signed_to_abs(displacement) as u16 ) }
                    else { self.reg.pc = self.reg.pc + ( displacement as u16 ) }
                    cycles += 5;
                }
                cycles += 7;
            },

            // JR NC,e
            0x30 => {
                if !self.reg.flags.c {
                    let displacement= self.bus.read_byte(self.reg.pc + 1);
                    if bit::get(displacement, 7) { self.reg.pc = self.reg.pc - ( signed_to_abs(displacement) as u16 ) }
                    else { self.reg.pc = self.reg.pc + ( displacement as u16 ) }
                    cycles += 5;
                }
                cycles += 7;
            },

            // JR Z,e
            0x28 => {
                if self.reg.flags.z {
                    let displacement= self.bus.read_byte(self.reg.pc + 1);
                    if bit::get(displacement, 7) { self.reg.pc = self.reg.pc - ( signed_to_abs(displacement) as u16 ) }
                    else { self.reg.pc = self.reg.pc + ( displacement as u16 ) }
                    cycles += 5;
                }
                cycles += 7;
            },

            // JR NZ,e
            0x20 => {
                if !self.reg.flags.z {
                    let displacement= self.bus.read_byte(self.reg.pc + 1);
                    if bit::get(displacement, 7) { self.reg.pc = self.reg.pc - ( signed_to_abs(displacement) as u16 ) }
                    else { self.reg.pc = self.reg.pc + ( displacement as u16 ) }
                    cycles += 5;
                }
                cycles += 7;
            },

            // JP (HL)
            0xE9 => { self.reg.pc = self.reg.get_hl(); },

            // DJNZ, e
            0x10 => {
                self.reg.b = (self.reg.b).wrapping_sub(1);
                if self.reg.b != 0 {
                    let displacement= self.bus.read_byte(self.reg.pc + 1);
                    if bit::get(displacement, 7) { self.reg.pc = self.reg.pc - ( signed_to_abs(displacement) as u16 ) }
                    else { self.reg.pc = self.reg.pc + ( displacement as u16 ) }
                    cycles += 5;
                }
                cycles += 8;
            }

            // Call and Return Group
            // CALL nn
            0xCD => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                self.call_stack_push();
                self.reg.pc = addr;
            },

            // CALL C,nn
            0xDC => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if self.reg.flags.c {
                    self.call_stack_push();
                    self.reg.pc = addr;
                    cycles += 7;
                } else { self.reg.pc += 3 }
            },

            // CALL NC,nn
            0xD4 => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if !self.reg.flags.c {
                    self.call_stack_push();
                    self.reg.pc = addr;
                    cycles += 7;
                } else { self.reg.pc += 3 }
            },

            // CALL Z,nn
            0xCC => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if self.reg.flags.z {
                    self.call_stack_push();
                    self.reg.pc = addr;
                    cycles += 7;
                } else { self.reg.pc += 3 }
            },

            // CALL NZ,nn
            0xC4 => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if !self.reg.flags.z {
                    self.call_stack_push();
                    self.reg.pc = addr;
                    cycles += 7;
                 } else { self.reg.pc += 3 }
            },

            // CALL M,nn
            0xFC => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if self.reg.flags.s {
                    self.call_stack_push();
                    self.reg.pc = addr;
                    cycles += 7;
                } else { self.reg.pc += 3 }
            },

            // CALL P,nn
            0xF4 => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if !self.reg.flags.s {
                    self.call_stack_push();
                    self.reg.pc = addr;
                    cycles += 7;
                } else { self.reg.pc += 3 }
            },

            // CALL PE,nn
            0xEC => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if self.reg.flags.p {
                    self.call_stack_push();
                    self.reg.pc = addr;
                    cycles += 7;
                } else { self.reg.pc += 3 }
            },

            // CALL PO,nn
            0xE4 => {
                let addr = self.bus.read_word(self.reg.pc + 1);
                if !self.reg.flags.p {
                    self.call_stack_push();
                    self.reg.pc = addr;
                    cycles += 7;
                } else { self.reg.pc += 3 }
            },

            // RET
            0xC9 => self.call_stack_pop(),

            // RET C
            0xD8 => if self.reg.flags.c { self.call_stack_pop(); cycles += 6; } else { self.reg.pc +=1; },

            // RET NC
            0xD0 => if !self.reg.flags.c { self.call_stack_pop(); cycles += 6; } else { self.reg.pc +=1; },

            // RET Z
            0xC8 => if self.reg.flags.z { self.call_stack_pop(); cycles += 6; } else { self.reg.pc +=1; },

            // RET NZ
            0xC0 => if !self.reg.flags.z { self.call_stack_pop(); cycles += 6; } else { self.reg.pc +=1; },

            // RET M
            0xF8 => if self.reg.flags.s { self.call_stack_pop(); cycles += 6; } else { self.reg.pc +=1; },

            // RET P
            0xF0 => if !self.reg.flags.s { self.call_stack_pop(); cycles += 6; } else { self.reg.pc +=1; },

            // RET PE
            0xE8 => if self.reg.flags.p { self.call_stack_pop(); cycles += 6; } else { self.reg.pc +=1; },

            // RET PO
            0xE0 => if !self.reg.flags.p { self.call_stack_pop(); cycles += 6; } else { self.reg.pc +=1; },

            // RST 0
            0xC7 => {
                match self.int {
                    Some(_) => self.interrupt_stack_push(),
                    None => { self.reg.pc +=1; self.interrupt_stack_push(); }
                }
                self.reg.pc = 0x0000;
            },

            // RST 08
            0xCF => {
                match self.int {
                    Some(_) => self.interrupt_stack_push(),
                    None => { self.reg.pc +=1; self.interrupt_stack_push(); }
                }
                self.reg.pc = 0x0008;
            },

            // RST 10
            0xD7 => {
                match self.int {
                    Some(_) => self.interrupt_stack_push(),
                    None => { self.reg.pc +=1; self.interrupt_stack_push(); }
                }
                self.reg.pc = 0x0010;
            },

            // RST 18
            0xDF => {
                match self.int {
                    Some(_) => self.interrupt_stack_push(),
                    None => { self.reg.pc +=1; self.interrupt_stack_push(); }
                }
                self.reg.pc = 0x0018;
            },

            // RST 20
            0xE7 => {
                match self.int {
                    Some(_) => self.interrupt_stack_push(),
                    None => { self.reg.pc +=1; self.interrupt_stack_push(); }
                }
                self.reg.pc = 0x0020;
            },

            // RST 28
            0xEF => {
                match self.int {
                    Some(_) => self.interrupt_stack_push(),
                    None => { self.reg.pc +=1; self.interrupt_stack_push(); }
                }
                self.reg.pc = 0x0028;
            },

            // RST 30
            0xF7 => {
                match self.int {
                    Some(_) => self.interrupt_stack_push(),
                    None => { self.reg.pc +=1; self.interrupt_stack_push(); }
                }
                self.reg.pc = 0x0030;
            },

            // RST 38
            0xFF => {
                match self.int {
                    Some(_) => self.interrupt_stack_push(),
                    None => { self.reg.pc +=1; self.interrupt_stack_push(); }
                }
                self.reg.pc = 0x0038;
            },

            // Input and Output Group
            // IN A,(n)
            0xDB => {
                let port = self.bus.read_byte(self.reg.pc + 1);
                self.reg.a = self.get_io(port);
                if self.debug.instr_in { println!("IN {:#04X} from device {:#04X}", self.reg.a, port) }
            },

            // OUT (n),A
            0xD3 => {
                let port = self.bus.read_byte(self.reg.pc + 1);
                self.set_io(port, self.reg.a);
                if self.debug.instr_in { println!("OUT {:#04X} sent to device {:#04X}", self.reg.a, port) }
            },

            _ => {
                if self.debug.unknw_instr { self.debug.string = format!("{:#04X}", opcode); }
                cycles = 0xFF;
            },

        }

        match opcode {
            0xC3 | 0xDA | 0xD2 | 0xCA | 0xC2 | 0xFA | 0xF2 | 0xEA |
            0xE2 | 0xE9 | 0xCD | 0xDC | 0xD4 | 0xCC | 0xC4 | 0xFC |
            0xF4 | 0xEC | 0xE4 | 0xC9 | 0xD8 | 0xD0 | 0xC8 | 0xC0 |
            0xF8 | 0xF0 | 0xE8 | 0xE0 | 0xC7 | 0xCF | 0xD7 | 0xDF |
            0xE7 | 0xEF | 0xF7 | 0xFF | 0x76 => {},
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E |
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6  | 0xF6 | 0xEE | 0xFE |
            0x18 | 0x38 | 0x30 | 0x28 | 0x20 | 0x10 | 0xDB | 0xD3 => self.reg.pc += 2,
            0x32 | 0x01 | 0x11 | 0x21 | 0x31 | 0x2A | 0x22 | 0x3A => self.reg.pc += 3,
            _ => self.reg.pc +=1,
        }

        if self.debug.opcode == true { self.debug.string = format!("{:#04X}", opcode) }

        cycles
    }
}

// Utility & debug functions / structs

// Check addition overflow
fn check_add_overflow(n1: u8, n2: u8) -> bool {
    let r = (n1 as i8).overflowing_add(n2 as i8);
    r.1
}

// Check substraction overflow
fn check_sub_overflow(n1: u8, n2: u8) -> bool {
    let r = (n1 as i8).overflowing_sub(n2 as i8);
    r.1
}

// Converts a signed byte to its absolute value
pub fn signed_to_abs(n: u8) -> u8 {
    !n +1
}

pub struct Debug {
    pub unknw_instr: bool,
    pub opcode: bool,
    // IO MPSC Messages
    pub io: bool,
    // Data read by IN instruction
    pub instr_in: bool,
    pub string: String
}

impl Debug {
    pub fn new() -> Debug {
        Debug {
            unknw_instr: false,
            opcode: false,
            io: false,
            instr_in: false,
            string: String::new(),
        }
    }
}