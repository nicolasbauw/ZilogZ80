use crate::bit;
use crate::bus::Bus;
use crate::cycles::{CYCLES, CYCLES_CB, CYCLES_DD_FD, CYCLES_ED};
use crate::registers::Registers;
use std::time::SystemTime;

const EI_DELAY_COUNTDOWN_START: u8 = 2;

pub struct CPU {
    pub reg: Registers,
    pub alt: Registers,
    halt: bool,
    /// Last unhandled instruction encountered, for the host to pick up. A
    /// library shouldn't decide where its diagnostics go: on this machine,
    /// standard output is the debugger's console, and an unknown
    /// instruction inside a game loop would drown out everything else.
    unimplemented: Option<Unimplemented>,
    unimplemented_count: u64,
    int: Option<u8>,
    /// True only while executing the current opcode when it was just
    /// injected by an interrupt (mode 0/1), rather than read normally from
    /// memory. Not to be confused with `int.is_some()`, which only means
    /// "an interrupt is pending": an interrupt can stay pending (masked by
    /// DI) while a completely different RST instruction, a real one read
    /// from memory, executes — RST handlers must increment PC in that case,
    /// not skip it.
    interrupt_acknowledge: bool,
    nmi: bool,
    im: u8,
    iff1: bool,
    iff2: bool,
    ei_instr_delay: u8,
    slice_duration: u32,
    // Defaults to 35000 cycles per 16ms slice (2.1 Mhz).
    // cycles = clock speed in Hz / required frames-per-second
    slice_max_cycles: u32,
    slice_current_cycles: u32,
    slice_start_time: SystemTime,
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

impl CPU {
    /// Creates a new CPU instance. 'Size' will be its top address.
    pub fn new() -> CPU {
        CPU {
            reg: Registers::new(),
            alt: Registers::new(),
            halt: false,
            unimplemented: None,
            unimplemented_count: 0,
            int: None,
            interrupt_acknowledge: false,
            nmi: false,
            im: 0,
            iff1: false,
            iff2: false,
            ei_instr_delay: 0,
            slice_duration: 16,
            slice_max_cycles: 35000,
            slice_current_cycles: 0,
            slice_start_time: SystemTime::now(),
        }
    }

    // --- Interrupt getters ---

    /// Returns the state of the first interrupt flip-flop (IFF1)
    pub fn iff1(&self) -> bool {
        self.iff1
    }

    /// Returns the state of the second interrupt flip-flop (IFF2)
    pub fn iff2(&self) -> bool {
        self.iff2
    }

    /// Returns the current interrupt mode (0, 1 or 2)
    pub fn im(&self) -> u8 {
        self.im
    }

    /// Whether the CPU is currently in the HALT state
    pub fn is_halted(&self) -> bool {
        self.halt
    }

    /// Whether a maskable interrupt (INT) is pending
    pub fn has_pending_int(&self) -> bool {
        self.int.is_some()
    }

    /// Whether a non-maskable interrupt (NMI) is pending
    pub fn has_pending_nmi(&self) -> bool {
        self.nmi
    }

    /// Creates a maskable interrupt request
    pub fn int_request(&mut self, byte: u8) {
        self.int = Some(byte);
    }

    /// Cancels a pending maskable interrupt request, as if the device that
    /// raised it had withdrawn the request before the CPU could acknowledge
    /// it. Symmetric with [`CPU::int_request`].
    ///
    /// Real hardware does this: on the Amstrad CPC, writing to the Gate
    /// Array's mode/ROM register with bit 4 set clears the pending interrupt
    /// request outright, in addition to resetting the interrupt divider.
    pub fn int_cancel(&mut self) {
        self.int = None;
    }

    /// Creates a non-maskable interrupt request
    pub fn nmi_request(&mut self) {
        self.nmi = true;
    }

    /// Shortcut to reg.flags.to_byte()
    /// Picks up the last unhandled instruction encountered, and forgets it.
    ///
    /// Meant to be polled after `execute()`: the host then chooses to
    /// display it, log it, or stop on it.
    pub fn take_unimplemented(&mut self) -> Option<Unimplemented> {
        self.unimplemented.take()
    }

    /// Total number of unhandled instructions since startup, including
    /// those that were never picked up: nothing disappears silently.
    pub fn unimplemented_count(&self) -> u64 {
        self.unimplemented_count
    }

    pub fn flags(&self) -> u8 {
        self.reg.flags.to_byte()
    }

    /// Returns true when maskable interrupts are currently enabled and ready to be accepted.
    /// Per Z80 spec, EI takes effect only after the instruction following EI completes, so
    /// `ei_instr_delay` must be zero (the one-instruction delay has expired).
    fn maskable_interrupts_enabled(&self) -> bool {
        self.iff1 && self.ei_instr_delay == 0
    }

    fn has_pending_maskable_interrupt(&self) -> bool {
        self.maskable_interrupts_enabled() && self.int.is_some()
    }

    fn interrupt_pending_during_instruction(&self) -> bool {
        self.iff1 && self.ei_instr_delay > 0 && self.int.is_some()
    }

    /// Fetches and executes one instruction from (pc). Returns consumed clock cycles.
    pub fn execute<B: Bus>(&mut self, bus: &mut B) -> u32 {
        // Interrupt request must stay latched while masked (DI/EI delay) and be cleared only when acknowledged.
        let mut clear_int_request = false;

        let has_pending_maskable_interrupt = self.has_pending_maskable_interrupt();
        if self.halt {
            if self.nmi || has_pending_maskable_interrupt {
                self.halt = false;
                // HALT deliberately leaves PC on its own opcode while the
                // CPU waits. It must therefore be advanced the moment an
                // interrupt takes the CPU out of this state, so that the
                // return address pushed is the NEXT instruction: otherwise
                // the handler's RET lands back on the HALT, trapping the
                // program there forever, one interrupt after another.
                self.reg.pc = self.reg.pc.wrapping_add(1);
            } else {
                // A Z80 in HALT keeps re-reading its own opcode while
                // waiting for the interrupt: each pass is a real M1 cycle,
                // so R advances even though nothing else happens.
                self.bump_r();
                return 4;
            }
        };

        // Non maskable interrupt requested ? IFF1 is saved into IFF2 (RETN restores it), PC is pushed
        // and execution restarts at 0x0066. The acknowledge cycle is 11 T-states and consumes the whole
        // call: the first instruction of the handler is fetched by the next execute() call, not by this one.
        if self.nmi {
            self.bump_r();
            self.iff2 = self.iff1;
            self.iff1 = false;
            self.interrupt_stack_push(bus);
            self.reg.pc = 0x0066;
            self.reg.wz = self.reg.pc;
            self.nmi = false;
            return 11;
        }

        let maskable_interrupts_enabled = self.maskable_interrupts_enabled();
        let has_pending_maskable_interrupt = self.has_pending_maskable_interrupt();

        // Accepting any maskable interrupt disables further maskable interrupts (IFF1 = IFF2 = false).
        // Interrupt nesting is only possible if the ISR explicitly calls EI to re-enable them.
        if has_pending_maskable_interrupt {
            self.iff1 = false;
            self.iff2 = false;
        }

        // Interrupt requested in interrupt mode 1 ? Restart at address 0038h (opcode 0xFF)
        if has_pending_maskable_interrupt && self.im == 1 {
            self.int = Some(0xFF);
        };

        // Interrupt requested in interrupt mode 2 ? Push PC onto the stack, build jump address and jump to that address.
        // The acknowledge cycle is 19 T-states and consumes the whole call: the first instruction of the
        // service routine is fetched by the next execute() call, not by this one.
        if has_pending_maskable_interrupt
            && self.im == 2
            && let Some(vector) = self.int
        {
            self.bump_r();
            self.interrupt_stack_push(bus);
            let addr = ((self.reg.i as u16) << 8) | (vector as u16);
            self.reg.pc = bus.read_word(addr);
            self.reg.wz = self.reg.pc;
            self.int = None;
            return 19;
        };

        // We retrieve the opcode, wether it comes from an interrupt request or normal fetch
        self.interrupt_acknowledge = false;
        let opcode = if maskable_interrupts_enabled {
            match self.int {
                None => bus.read_byte(self.reg.pc),
                Some(o) => {
                    clear_int_request = true;
                    self.interrupt_acknowledge = true;
                    o
                }
            }
        } else {
            bus.read_byte(self.reg.pc)
        };

        self.bump_r();
        let mut cycles = match opcode {
            0xDD | 0xFD | 0xED | 0xCB => self.execute_2bytes(bus),
            _ => self.execute_1byte(bus, opcode),
        };

        // In IM 0 and IM 1 the opcode comes from the interrupting device instead of memory:
        // the M1 acknowledge cycle is lengthened by two wait states. An IM 1 acknowledge
        // (a forced RST 38) therefore takes 11 + 2 = 13 T-states.
        if self.interrupt_acknowledge {
            cycles += 2;
        }

        if clear_int_request {
            self.int = None;
        }
        if self.ei_instr_delay > 0 {
            self.ei_instr_delay -= 1;
        }
        cycles
    }

    /// Fetches and executes one instruction from (pc). Returns the sleep time when slice_max_cycles is reached.
    pub fn execute_timed<B: Bus>(&mut self, bus: &mut B) -> Option<u32> {
        let mut sleep_time: Option<u32> = None;
        if self.slice_current_cycles > self.slice_max_cycles {
            self.slice_current_cycles = 0;
            // d = time taken to execute the slice_max_cycles
            if let Ok(d) = self.slice_start_time.elapsed() {
                sleep_time = Some(self.slice_duration.saturating_sub(d.as_millis() as u32));
                self.slice_start_time = SystemTime::now();
            }
        }
        let cycles = self.execute(bus);
        self.slice_current_cycles += cycles;
        sleep_time
    }

    /// Sets CPU frequency (MHz). Effective only when used with execute_timed().
    /// ```rust
    /// use zilog_z80::cpu::CPU;
    /// let mut c = CPU::new();
    /// c.set_freq(1.7);            // CPU will run at 1.7 Mhz
    /// ```
    pub fn set_freq(&mut self, f: f32) {
        let cycles = (f * 1000000_f32) / (1000 / self.slice_duration) as f32;
        self.slice_max_cycles = cycles as u32;
    }

    /// Sets slice duration (in milliseconds) for timed execution. Typically 16 for 60 Hz, 20 for 50 Hz screen refresh.
    pub fn set_slice_duration(&mut self, slice_duration: u32) {
        self.slice_duration = slice_duration;
    }

    fn execute_1byte<B: Bus>(&mut self, bus: &mut B, opcode: u8) -> u32 {
        let mut cycles = CYCLES[opcode as usize].into();

        // Saving current PC for debug output
        //let pc = self.reg.pc;

        match opcode {
            // 8-Bit Load Group
            // LD r,r'      LD r,(HL)
            0x40 => {}                       // LD B,B
            0x41 => self.reg.b = self.reg.c, // LD B,C
            0x42 => self.reg.b = self.reg.d, // LD B,D
            0x43 => self.reg.b = self.reg.e, // LD B,E
            0x44 => self.reg.b = self.reg.h, // LD B,H
            0x45 => self.reg.b = self.reg.l, // LD B,L
            0x46 => {
                // LD B,(HL)
                let addr = self.reg.get_hl();
                self.reg.b = bus.read_byte(addr)
            }
            0x47 => self.reg.b = self.reg.a, // LD B,A

            0x48 => self.reg.c = self.reg.b, // LD C,B
            0x49 => {}                       // LD C,C
            0x4A => self.reg.c = self.reg.d, // LD C,D
            0x4B => self.reg.c = self.reg.e, // LD C,E
            0x4C => self.reg.c = self.reg.h, // LD C,H
            0x4D => self.reg.c = self.reg.l, // LD C,L
            0x4E => {
                // LD C,(HL)
                let addr = self.reg.get_hl();
                self.reg.c = bus.read_byte(addr)
            }
            0x4F => self.reg.c = self.reg.a, // LD C,A

            0x50 => self.reg.d = self.reg.b, // LD D,B
            0x51 => self.reg.d = self.reg.c, // LD D,C
            0x52 => {}                       // LD D,D
            0x53 => self.reg.d = self.reg.e, // LD D,E
            0x54 => self.reg.d = self.reg.h, // LD D,H
            0x55 => self.reg.d = self.reg.l, // LD D,L
            0x56 => {
                // LD D,(HL)
                let addr = self.reg.get_hl();
                self.reg.d = bus.read_byte(addr)
            }
            0x57 => self.reg.d = self.reg.a, // LD D,A

            0x58 => self.reg.e = self.reg.b, // LD E,B
            0x59 => self.reg.e = self.reg.c, // LD E,C
            0x5A => self.reg.e = self.reg.d, // LD E,D
            0x5B => {}                       // LD E,E
            0x5C => self.reg.e = self.reg.h, // LD E,H
            0x5D => self.reg.e = self.reg.l, // LD E,L
            0x5E => {
                // LD E,(HL)
                let addr = self.reg.get_hl();
                self.reg.e = bus.read_byte(addr)
            }
            0x5F => self.reg.e = self.reg.a, // LD E,A

            0x60 => self.reg.h = self.reg.b, // LD H,B
            0x61 => self.reg.h = self.reg.c, // LD H,C
            0x62 => self.reg.h = self.reg.d, // LD H,D
            0x63 => self.reg.h = self.reg.e, // LD H,E
            0x64 => {}                       // LD H,H
            0x65 => self.reg.h = self.reg.l, // LD H,L
            0x66 => {
                // LD H,(HL)
                let addr = self.reg.get_hl();
                self.reg.h = bus.read_byte(addr)
            }
            0x67 => self.reg.h = self.reg.a, // LD H,A

            0x68 => self.reg.l = self.reg.b, // LD L,B
            0x69 => self.reg.l = self.reg.c, // LD L,C
            0x6A => self.reg.l = self.reg.d, // LD L,D
            0x6B => self.reg.l = self.reg.e, // LD L,E
            0x6C => self.reg.l = self.reg.h, // LD L,H
            0x6D => {}                       // LD L,L
            0x6E => {
                // LD L,(HL)
                let addr = self.reg.get_hl();
                self.reg.l = bus.read_byte(addr)
            }
            0x6F => self.reg.l = self.reg.a, // LD L,A

            0x78 => self.reg.a = self.reg.b, // LD A,B
            0x79 => self.reg.a = self.reg.c, // LD A,C
            0x7A => self.reg.a = self.reg.d, // LD A,D
            0x7B => self.reg.a = self.reg.e, // LD A,E
            0x7C => self.reg.a = self.reg.h, // LD A,H
            0x7D => self.reg.a = self.reg.l, // LD A,L
            0x7E => {
                // LD A,(HL)
                let addr = self.reg.get_hl();
                self.reg.a = bus.read_byte(addr)
            }
            0x7F => {} // LD A,A

            // LD (HL),r
            0x70 => {
                // LD (HL), B
                let addr = self.reg.get_hl();
                bus.write_byte(addr, self.reg.b)
            }
            0x71 => {
                // LD (HL), C
                let addr = self.reg.get_hl();
                bus.write_byte(addr, self.reg.c)
            }
            0x72 => {
                // LD (HL), D
                let addr = self.reg.get_hl();
                bus.write_byte(addr, self.reg.d)
            }
            0x73 => {
                // LD (HL), E
                let addr = self.reg.get_hl();
                bus.write_byte(addr, self.reg.e)
            }
            0x74 => {
                // LD (HL), H
                let addr = self.reg.get_hl();
                bus.write_byte(addr, self.reg.h)
            }
            0x75 => {
                // LD (HL), L
                let addr = self.reg.get_hl();
                bus.write_byte(addr, self.reg.l)
            }

            0x77 => {
                // LD (HL), A
                let addr = self.reg.get_hl();
                bus.write_byte(addr, self.reg.a)
            }

            // LD r,n
            0x06 => {
                // LD B,n
                let data = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.reg.b = data;
            }
            0x0E => {
                // LD C,n
                let data = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.reg.c = data;
            }
            0x16 => {
                // LD D,n
                let data = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.reg.d = data;
            }
            0x1E => {
                // LD E,n
                let data = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.reg.e = data;
            }
            0x26 => {
                // LD H,n
                let data = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.reg.h = data;
            }
            0x2E => {
                // LD L,n
                let data = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.reg.l = data;
            }
            0x36 => {
                // LD (HL),n
                let data = bus.read_byte(self.reg.pc.wrapping_add(1));
                let addr = self.reg.get_hl();
                bus.write_byte(addr, data);
            }
            0x3E => {
                // LD A,n
                let data = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.reg.a = data;
            }

            // LD A,(BC)
            0x0A => {
                let addr = self.reg.get_bc();
                self.reg.a = bus.read_byte(addr);
                self.wz_after(addr);
            }

            // LD A,(DE)
            0x1A => {
                let addr = self.reg.get_de();
                self.reg.a = bus.read_byte(addr);
                self.wz_after(addr);
            }

            // LD A,(nn)
            0x3A => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.a = bus.read_byte(addr);
                self.wz_after(addr);
            }

            // LD (BC),A
            0x02 => {
                let addr = self.reg.get_bc();
                bus.write_byte(addr, self.reg.a);
                self.wz_after_write_a(addr);
            }

            // LD (DE),A
            0x12 => {
                let addr = self.reg.get_de();
                bus.write_byte(addr, self.reg.a);
                self.wz_after_write_a(addr);
            }

            // LD (nn),A
            0x32 => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                bus.write_byte(addr, self.reg.a);
                self.wz_after_write_a(addr);
            }

            // 16-Bit Load Group
            // LD dd,nn
            0x01 => {
                // LD BC,nn
                let d16 = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.set_bc(d16);
            }
            0x11 => {
                // LD DE,nn
                let d16 = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.set_de(d16);
            }
            0x21 => {
                // LD HL,nn
                let d16 = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.set_hl(d16);
            }
            0x31 => {
                // LD SP,nn
                let d16 = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.sp = d16;
            }

            // LD HL,(nn)
            0x2A => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                let d = bus.read_word(addr);
                self.reg.set_hl(d);
                self.wz_after(addr);
            }

            // LD (nn),HL
            0x22 => {
                let d = self.reg.get_hl();
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                bus.write_word(addr, d);
                self.wz_after(addr);
            }

            // LD SP,HL
            0xF9 => self.reg.sp = self.reg.get_hl(),

            // PUSH qq
            0xC5 => {
                // PUSH BC
                self.reg.sp = self.reg.sp.wrapping_sub(2);
                bus.write_word(self.reg.sp, self.reg.get_bc());
            }
            0xD5 => {
                // PUSH DE
                self.reg.sp = self.reg.sp.wrapping_sub(2);
                bus.write_word(self.reg.sp, self.reg.get_de());
            }
            0xE5 => {
                // PUSH HL
                self.reg.sp = self.reg.sp.wrapping_sub(2);
                bus.write_word(self.reg.sp, self.reg.get_hl());
            }
            0xF5 => {
                // PUSH AF
                self.reg.sp = self.reg.sp.wrapping_sub(2);
                bus.write_byte(self.reg.sp, self.reg.flags.to_byte());
                bus.write_byte(self.reg.sp.wrapping_add(1), self.reg.a);
            }

            // POP qq
            0xC1 => {
                // POP BC
                self.reg.set_bc(bus.read_word(self.reg.sp));
                self.reg.sp = self.reg.sp.wrapping_add(2);
            }

            0xD1 => {
                // POP DE
                self.reg.set_de(bus.read_word(self.reg.sp));
                self.reg.sp = self.reg.sp.wrapping_add(2);
            }

            0xE1 => {
                // POP HL
                self.reg.set_hl(bus.read_word(self.reg.sp));
                self.reg.sp = self.reg.sp.wrapping_add(2);
            }

            0xF1 => {
                // POP AF
                self.reg.a = bus.read_byte(self.reg.sp.wrapping_add(1));
                let bflags = bus.read_byte(self.reg.sp);
                self.reg.flags.set_from_byte(bflags);
                self.reg.sp = self.reg.sp.wrapping_add(2);
            }

            // Exchange, Block Transfer, and Search Group
            // EX DE,HL
            0xEB => {
                let de = self.reg.get_de();
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
                let pointed_by_sp = bus.read_word(self.reg.sp);
                let hl = self.reg.get_hl();
                bus.write_word(self.reg.sp, hl);
                self.reg.set_hl(pointed_by_sp);
                self.reg.wz = pointed_by_sp;
            }

            // 8-Bit Arithmetic Group
            // ADD A,r
            0x80 => self.add(self.reg.b), // ADD A,B
            0x81 => self.add(self.reg.c), // ADD A,C
            0x82 => self.add(self.reg.d), // ADD A,D
            0x83 => self.add(self.reg.e), // ADD A,E
            0x84 => self.add(self.reg.h), // ADD A,H
            0x85 => self.add(self.reg.l), // ADD A,L
            0x86 => {
                // ADD (HL)
                let addr = self.reg.get_hl();
                let n = bus.read_byte(addr);
                self.add(n)
            }
            0x87 => self.add(self.reg.a), // ADD A,A

            // ADD A,n
            0xC6 => {
                let n = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.add(n);
            }

            // ADC A,r
            0x88 => self.adc(self.reg.b), // ADC A,B
            0x89 => self.adc(self.reg.c), // ADC A,C
            0x8A => self.adc(self.reg.d), // ADC A,D
            0x8B => self.adc(self.reg.e), // ADC A,E
            0x8C => self.adc(self.reg.h), // ADC A,H
            0x8D => self.adc(self.reg.l), // ADC A,L
            0x8E => {
                // ADC A,(HL)
                let addr = self.reg.get_hl();
                let n = bus.read_byte(addr);
                self.adc(n)
            }
            0x8F => self.adc(self.reg.a), // ADC A,A

            // ADC a,n
            0xCE => {
                // ADC A,(HL)
                let n = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.adc(n)
            }

            // SUB s
            0x90 => self.sub(self.reg.b), // SUB A,B
            0x91 => self.sub(self.reg.c), // SUB A,C
            0x92 => self.sub(self.reg.d), // SUB A,D
            0x93 => self.sub(self.reg.e), // SUB A,E
            0x94 => self.sub(self.reg.h), // SUB A,H
            0x95 => self.sub(self.reg.l), // SUB A,L
            0x96 => {
                // SUB A,(HL)
                let addr = self.reg.get_hl();
                let n = bus.read_byte(addr);
                self.sub(n)
            }
            0x97 => self.sub(self.reg.a), // SUB A,A

            0xD6 => {
                // SUB A,n
                let n = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.sub(n);
            }

            // SBC A,s
            0x98 => self.sbc(self.reg.b), // SBC A,B
            0x99 => self.sbc(self.reg.c), // SBC A,C
            0x9A => self.sbc(self.reg.d), // SBC A,D
            0x9B => self.sbc(self.reg.e), // SBC A,E
            0x9C => self.sbc(self.reg.h), // SBC A,H
            0x9D => self.sbc(self.reg.l), // SBC A,L
            0x9E => {
                // SBC A,(HL)
                let addr = self.reg.get_hl();
                let n = bus.read_byte(addr);
                self.sbc(n)
            }
            0x9F => self.sbc(self.reg.a), // SBC A,A

            0xDE => {
                // SBC A,n
                let n = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.sbc(n);
            }

            // AND s
            0xA0 => self.and(self.reg.b), // AND B
            0xA1 => self.and(self.reg.c), // AND C
            0xA2 => self.and(self.reg.d), // AND D
            0xA3 => self.and(self.reg.e), // AND E
            0xA4 => self.and(self.reg.h), // AND H
            0xA5 => self.and(self.reg.l), // AND L
            0xA6 => {
                // AND (HL)
                let addr = self.reg.get_hl();
                let n = bus.read_byte(addr);
                self.and(n)
            }
            0xA7 => self.and(self.reg.a), // AND A

            0xE6 => {
                // AND n
                let n = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.and(n);
            }

            // OR s
            0xB0 => self.or(self.reg.b), // OR B
            0xB1 => self.or(self.reg.c), // OR C
            0xB2 => self.or(self.reg.d), // OR D
            0xB3 => self.or(self.reg.e), // OR E
            0xB4 => self.or(self.reg.h), // OR H
            0xB5 => self.or(self.reg.l), // OR L
            0xB6 => {
                // OR (HL)
                let addr = self.reg.get_hl();
                let n = bus.read_byte(addr);
                self.or(n)
            }
            0xB7 => self.or(self.reg.a), // OR A

            0xF6 => {
                // OR n
                let n = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.or(n);
            }

            // XOR s
            0xA8 => self.xor(self.reg.b), // XOR B
            0xA9 => self.xor(self.reg.c), // XOR C
            0xAA => self.xor(self.reg.d), // XOR D
            0xAB => self.xor(self.reg.e), // XOR E
            0xAC => self.xor(self.reg.h), // XOR H
            0xAD => self.xor(self.reg.l), // XOR L
            0xAE => {
                // XOR (HL)
                let addr = self.reg.get_hl();
                let n = bus.read_byte(addr);
                self.xor(n)
            }
            0xAF => self.xor(self.reg.a), // XOR A

            0xEE => {
                // XOR n
                let n = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.xor(n);
            }

            // CMP s
            0xB8 => self.cp(self.reg.b), // CP B
            0xB9 => self.cp(self.reg.c), // CP C
            0xBA => self.cp(self.reg.d), // CP D
            0xBB => self.cp(self.reg.e), // CP E
            0xBC => self.cp(self.reg.h), // CP H
            0xBD => self.cp(self.reg.l), // CP L
            0xBE => {
                // CP (HL)
                let addr = self.reg.get_hl();
                let n = bus.read_byte(addr);
                self.cp(n)
            }
            0xBF => self.cp(self.reg.a), // CP A

            0xFE => {
                // CP n
                let n = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.cp(n);
            }

            // INC r
            0x04 => self.reg.b = self.inc(self.reg.b), // INC B
            0x0C => self.reg.c = self.inc(self.reg.c), // INC C
            0x14 => self.reg.d = self.inc(self.reg.d), // INC D
            0x1C => self.reg.e = self.inc(self.reg.e), // INC E
            0x24 => self.reg.h = self.inc(self.reg.h), // INC H
            0x2C => self.reg.l = self.inc(self.reg.l), // INC L
            0x34 => {
                // INC (HL)
                let addr = self.reg.get_hl();
                let r = self.inc(bus.read_byte(addr));
                bus.write_byte(addr, r);
            }
            0x3C => self.reg.a = self.inc(self.reg.a), // INC A

            // DEC m
            0x05 => self.reg.b = self.dec(self.reg.b), // DEC B
            0x0D => self.reg.c = self.dec(self.reg.c), // DEC C
            0x15 => self.reg.d = self.dec(self.reg.d), // DEC D
            0x1D => self.reg.e = self.dec(self.reg.e), // DEC E
            0x25 => self.reg.h = self.dec(self.reg.h), // DEC H
            0x2D => self.reg.l = self.dec(self.reg.l), // DEC L
            0x35 => {
                // DEC (HL)
                let addr = self.reg.get_hl();
                let r = self.dec(bus.read_byte(addr));
                bus.write_byte(addr, r);
            }
            0x3D => self.reg.a = self.dec(self.reg.a), // DEC A

            // General-Purpose Arithmetic and CPU Control Groups
            // DAA
            0x27 => self.daa(),

            // CPL
            0x2F => {
                self.reg.a = !self.reg.a;
                self.reg.flags.h = true;
                self.reg.flags.n = true;
                self.reg.flags.set_undocumented_from(self.reg.a);
            }

            // CCF
            0x3F => {
                self.reg.flags.h = self.reg.flags.c;
                self.reg.flags.c = !self.reg.flags.c;
                self.reg.flags.n = false;
                // On a Zilog NMOS (the one in the CPC), SCF/CCF take their
                // two undocumented flags directly from A.
                self.reg.flags.set_undocumented_from(self.reg.a);
            }

            // SCF
            0x37 => {
                self.reg.flags.c = true;
                self.reg.flags.h = false;
                self.reg.flags.n = false;
                self.reg.flags.set_undocumented_from(self.reg.a);
            }

            // NOP
            0x00 => {}

            // HALT
            0x76 => self.halt = true,

            // DI
            0xF3 => {
                self.iff1 = false;
                self.iff2 = false;
                self.ei_instr_delay = 0;
            }

            // EI
            0xFB => {
                self.iff1 = true;
                self.iff2 = true;
                self.ei_instr_delay = EI_DELAY_COUNTDOWN_START;
            }

            // 16-Bit Arithmetic Group
            // ADD HL,ss
            0x09 => {
                // ADD HL,BC
                let reg = self.reg.get_bc();
                let r = self.add_16(self.reg.get_hl(), reg);
                self.reg.set_hl(r);
            }
            0x19 => {
                // ADD HL,DE
                let reg = self.reg.get_de();
                let r = self.add_16(self.reg.get_hl(), reg);
                self.reg.set_hl(r);
            }
            0x29 => {
                // ADD HL,HL
                let reg = self.reg.get_hl();
                let r = self.add_16(self.reg.get_hl(), reg);
                self.reg.set_hl(r);
            }
            0x39 => {
                // ADD HL,SP
                let reg = self.reg.sp;
                let r = self.add_16(self.reg.get_hl(), reg);
                self.reg.set_hl(r);
            }

            // INC ss
            0x03 => {
                // INC BC
                let r = self.reg.get_bc().wrapping_add(1);
                self.reg.set_bc(r);
            }

            0x13 => {
                // INC DE
                let r = self.reg.get_de().wrapping_add(1);
                self.reg.set_de(r);
            }

            0x23 => {
                // INC HL
                let r = self.reg.get_hl().wrapping_add(1);
                self.reg.set_hl(r);
            }

            0x33 => {
                // INC SP
                let r = self.reg.sp.wrapping_add(1);
                self.reg.sp = r;
            }

            // DEC ss
            0x0B => {
                // DEC BC
                let r = self.reg.get_bc().wrapping_sub(1);
                self.reg.set_bc(r);
            }

            0x1B => {
                // DEC DE
                let r = self.reg.get_de().wrapping_sub(1);
                self.reg.set_de(r);
            }

            0x2B => {
                // DEC HL
                let r = self.reg.get_hl().wrapping_sub(1);
                self.reg.set_hl(r);
            }

            0x3B => {
                // DEC SP
                let r = self.reg.sp.wrapping_sub(1);
                self.reg.sp = r;
            }

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
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                self.reg.pc = addr;
            }

            // JP C,nn
            0xDA => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if self.reg.flags.c {
                    self.reg.pc = addr;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // JP NC,nn
            0xD2 => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if !self.reg.flags.c {
                    self.reg.pc = addr;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // JP Z,nn
            0xCA => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if self.reg.flags.z {
                    self.reg.pc = addr;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // JP NZ,nn
            0xC2 => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if !self.reg.flags.z {
                    self.reg.pc = addr;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // JP M,nn
            0xFA => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if self.reg.flags.s {
                    self.reg.pc = addr;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // JP P,nn
            0xF2 => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if !self.reg.flags.s {
                    self.reg.pc = addr;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // JP PE,nn
            0xEA => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if self.reg.flags.p {
                    self.reg.pc = addr;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // JP PO,nn
            0xE2 => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if !self.reg.flags.p {
                    self.reg.pc = addr;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // JR e
            0x18 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(1));
                self.reg.pc = self.relative_target(displacement);
            }

            // JR C,e
            0x38 => {
                if self.reg.flags.c {
                    let displacement = bus.read_byte(self.reg.pc.wrapping_add(1));
                    self.reg.pc = self.relative_target(displacement);
                    cycles += 5;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2)
                }
                cycles += 7;
            }

            // JR NC,e
            0x30 => {
                if !self.reg.flags.c {
                    let displacement = bus.read_byte(self.reg.pc.wrapping_add(1));
                    self.reg.pc = self.relative_target(displacement);
                    cycles += 5;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2)
                }
                cycles += 7;
            }

            // JR Z,e
            0x28 => {
                if self.reg.flags.z {
                    let displacement = bus.read_byte(self.reg.pc.wrapping_add(1));
                    self.reg.pc = self.relative_target(displacement);
                    cycles += 5;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2)
                }
                cycles += 7;
            }

            // JR NZ,e
            0x20 => {
                if !self.reg.flags.z {
                    let displacement = bus.read_byte(self.reg.pc.wrapping_add(1));
                    self.reg.pc = self.relative_target(displacement);
                    cycles += 5;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2)
                }
                cycles += 7;
            }

            // JP (HL)
            0xE9 => {
                self.reg.pc = self.reg.get_hl();
            }

            // DJNZ, e
            0x10 => {
                self.reg.b = (self.reg.b).wrapping_sub(1);
                if self.reg.b != 0 {
                    let displacement = bus.read_byte(self.reg.pc.wrapping_add(1));
                    self.reg.pc = self.relative_target(displacement);
                    cycles += 5;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2)
                }
                cycles += 8;
            }

            // Call and Return Group
            // CALL nn
            0xCD => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                self.call_stack_push(bus);
                self.reg.pc = addr;
            }

            // CALL C,nn
            0xDC => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if self.reg.flags.c {
                    self.call_stack_push(bus);
                    self.reg.pc = addr;
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // CALL NC,nn
            0xD4 => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if !self.reg.flags.c {
                    self.call_stack_push(bus);
                    self.reg.pc = addr;
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // CALL Z,nn
            0xCC => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if self.reg.flags.z {
                    self.call_stack_push(bus);
                    self.reg.pc = addr;
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // CALL NZ,nn
            0xC4 => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if !self.reg.flags.z {
                    self.call_stack_push(bus);
                    self.reg.pc = addr;
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // CALL M,nn
            0xFC => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if self.reg.flags.s {
                    self.call_stack_push(bus);
                    self.reg.pc = addr;
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // CALL P,nn
            0xF4 => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if !self.reg.flags.s {
                    self.call_stack_push(bus);
                    self.reg.pc = addr;
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // CALL PE,nn
            0xEC => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if self.reg.flags.p {
                    self.call_stack_push(bus);
                    self.reg.pc = addr;
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // CALL PO,nn
            0xE4 => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(1));
                self.reg.wz = addr;
                if !self.reg.flags.p {
                    self.call_stack_push(bus);
                    self.reg.pc = addr;
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(3)
                }
            }

            // RET
            0xC9 => self.call_stack_pop(bus),

            // RET C
            0xD8 => {
                if self.reg.flags.c {
                    self.call_stack_pop(bus);
                    cycles += 6;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                }
            }

            // RET NC
            0xD0 => {
                if !self.reg.flags.c {
                    self.call_stack_pop(bus);
                    cycles += 6;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                }
            }

            // RET Z
            0xC8 => {
                if self.reg.flags.z {
                    self.call_stack_pop(bus);
                    cycles += 6;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                }
            }

            // RET NZ
            0xC0 => {
                if !self.reg.flags.z {
                    self.call_stack_pop(bus);
                    cycles += 6;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                }
            }

            // RET M
            0xF8 => {
                if self.reg.flags.s {
                    self.call_stack_pop(bus);
                    cycles += 6;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                }
            }

            // RET P
            0xF0 => {
                if !self.reg.flags.s {
                    self.call_stack_pop(bus);
                    cycles += 6;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                }
            }

            // RET PE
            0xE8 => {
                if self.reg.flags.p {
                    self.call_stack_pop(bus);
                    cycles += 6;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                }
            }

            // RET PO
            0xE0 => {
                if !self.reg.flags.p {
                    self.call_stack_pop(bus);
                    cycles += 6;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                }
            }

            // RST 0
            0xC7 => {
                if self.interrupt_acknowledge {
                    self.interrupt_stack_push(bus);
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                    self.interrupt_stack_push(bus);
                }
                self.reg.pc = 0x0000;
                self.reg.wz = self.reg.pc;
            }

            // RST 08
            0xCF => {
                if self.interrupt_acknowledge {
                    self.interrupt_stack_push(bus);
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                    self.interrupt_stack_push(bus);
                }
                self.reg.pc = 0x0008;
                self.reg.wz = self.reg.pc;
            }

            // RST 10
            0xD7 => {
                if self.interrupt_acknowledge {
                    self.interrupt_stack_push(bus);
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                    self.interrupt_stack_push(bus);
                }
                self.reg.pc = 0x0010;
                self.reg.wz = self.reg.pc;
            }

            // RST 18
            0xDF => {
                if self.interrupt_acknowledge {
                    self.interrupt_stack_push(bus);
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                    self.interrupt_stack_push(bus);
                }
                self.reg.pc = 0x0018;
                self.reg.wz = self.reg.pc;
            }

            // RST 20
            0xE7 => {
                if self.interrupt_acknowledge {
                    self.interrupt_stack_push(bus);
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                    self.interrupt_stack_push(bus);
                }
                self.reg.pc = 0x0020;
                self.reg.wz = self.reg.pc;
            }

            // RST 28
            0xEF => {
                if self.interrupt_acknowledge {
                    self.interrupt_stack_push(bus);
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                    self.interrupt_stack_push(bus);
                }
                self.reg.pc = 0x0028;
                self.reg.wz = self.reg.pc;
            }

            // RST 30
            0xF7 => {
                if self.interrupt_acknowledge {
                    self.interrupt_stack_push(bus);
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                    self.interrupt_stack_push(bus);
                }
                self.reg.pc = 0x0030;
                self.reg.wz = self.reg.pc;
            }

            // RST 38
            0xFF => {
                if self.interrupt_acknowledge {
                    self.interrupt_stack_push(bus);
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                    self.interrupt_stack_push(bus);
                }
                self.reg.pc = 0x0038;
                self.reg.wz = self.reg.pc;
            }

            // OUT (n), A (Opcode 0xD3)
            0xD3 => {
                let n = bus.read_byte(self.reg.pc.wrapping_add(1));
                // The Z80's 16-bit I/O port: A on the high byte, n on the low byte.
                let port = ((self.reg.a as u16) << 8) | (n as u16);
                bus.write_io(port, self.reg.a);
                // OUT (n),A follows the same oddball rule as LD (nn),A.
                self.wz_after_write_a(port);
            }

            // IN A, (n) (Opcode 0xDB)
            0xDB => {
                let n = bus.read_byte(self.reg.pc.wrapping_add(1));
                let port = ((self.reg.a as u16) << 8) | (n as u16);
                self.reg.a = bus.read_io(port);
                self.wz_after(port);
            }

            _ => {
                // No one-byte instruction is missing today, but the
                // reported duration must still match the table: a fallback
                // that invents an execution time would silently throw off
                // every part of the machine clocked from it.
                self.record_unimplemented(bus, 1);
            }
        }

        match opcode {
            0xC3 | 0xDA | 0xD2 | 0xCA | 0xC2 | 0xFA | 0xF2 | 0xEA | 0xE2 | 0xE9 | 0xCD | 0xDC
            | 0xD4 | 0xCC | 0xC4 | 0xFC | 0xF4 | 0xEC | 0xE4 | 0xC9 | 0xD8 | 0xD0 | 0xC8 | 0xC0
            | 0xF8 | 0xF0 | 0xE8 | 0xE0 | 0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF
            | 0x76 | 0x18 | 0x38 | 0x30 | 0x28 | 0x20 | 0x10 => {}
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E | 0xC6 | 0xCE | 0xD6 | 0xDE
            | 0xE6 | 0xF6 | 0xEE | 0xFE | 0xDB | 0xD3 => self.reg.pc = self.reg.pc.wrapping_add(2),
            0x32 | 0x01 | 0x11 | 0x21 | 0x31 | 0x2A | 0x22 | 0x3A => {
                self.reg.pc = self.reg.pc.wrapping_add(3)
            }
            _ => self.reg.pc = self.reg.pc.wrapping_add(1),
        }

        cycles
    }

    /// Variant of `repeat_block` for `LDIR`/`LDDR`/`CPIR`/`CPDR`: as long as
    /// they keep repeating, they reload MEMPTR with their own opcode's
    /// address plus one, since the processor is about to read it again.
    /// The repeated block I/O forms don't follow this rule.
    fn repeat_block_wz(&mut self, again: bool) -> u32 {
        if again {
            self.reg.wz = self.reg.pc.wrapping_add(1);
        }
        self.repeat_block(again)
    }

    /// Ends one iteration of a repeating instruction (LDIR, CPIR, OTIR...).
    ///
    /// If the repetition must continue, PC steps back by two so the
    /// instruction gets replayed on the next call: that's exactly what the
    /// Z80 does, and it's what lets an interrupt slip in between two
    /// iterations. Returns the iteration's duration: 21 cycles while it
    /// repeats, 16 for the last one.
    fn repeat_block(&mut self, again: bool) -> u32 {
        if again {
            // PC will be advanced by two at the end of execution: undo that.
            self.reg.pc = self.reg.pc.wrapping_sub(2);
            21
        } else {
            16
        }
    }

    fn execute_2bytes<B: Bus>(&mut self, bus: &mut B) -> u32 {
        // Second M1 cycle of the prefixed form: the chip reads the byte
        // following CB/ED/DD/FD as a second opcode. The DD/FD CB forms
        // (execute_4bytes) add nothing more: on a real Z80, the
        // displacement and final byte of this four-byte form are plain
        // memory reads, not M1 cycles.
        self.bump_r();
        let opcode = bus.read_le_word(self.reg.pc);
        let mut cycles = match opcode & 0xFF00 {
            0xDD00 | 0xFD00 => CYCLES_DD_FD[(opcode & 0x00FF) as usize].into(),
            0xED00 => CYCLES_ED[(opcode & 0x00FF) as usize].into(),
            0xCB00 => CYCLES_CB[(opcode & 0x00FF) as usize].into(),
            _ => 0,
        };

        match opcode {
            // 4 bytes instructions
            0xDDCB | 0xFDCB => return self.execute_4bytes(bus),

            // 8-Bit Load Group
            // LD r,(IX+d)
            0xDD46 => {
                // LD B,(IX+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                self.reg.b = bus.read_byte(address)
            }
            0xDD4E => {
                // LD C,(IX+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                self.reg.c = bus.read_byte(address)
            }
            0xDD56 => {
                // LD D,(IX+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                self.reg.d = bus.read_byte(address)
            }
            0xDD5E => {
                // LD E,(IX+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                self.reg.e = bus.read_byte(address)
            }
            0xDD66 => {
                // LD H,(IX+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                self.reg.h = bus.read_byte(address)
            }
            0xDD6E => {
                // LD L,(IX+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                self.reg.l = bus.read_byte(address)
            }
            0xDD7E => {
                // LD A,(IX+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                self.reg.a = bus.read_byte(address)
            }

            // LD r,(IY+d)
            0xFD46 => {
                // LD B,(IY+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                self.reg.b = bus.read_byte(address)
            }
            0xFD4E => {
                // LD C,(IY+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                self.reg.c = bus.read_byte(address)
            }
            0xFD56 => {
                // LD D,(IY+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                self.reg.d = bus.read_byte(address)
            }
            0xFD5E => {
                // LD E,(IY+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                self.reg.e = bus.read_byte(address)
            }
            0xFD66 => {
                // LD H,(IY+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                self.reg.h = bus.read_byte(address)
            }
            0xFD6E => {
                // LD L,(IY+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                self.reg.l = bus.read_byte(address)
            }
            0xFD7E => {
                // LD A,(IY+d)
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                self.reg.a = bus.read_byte(address)
            }

            // LD (IX+d),r
            0xDD70 => {
                // LD (IX+d),B
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                bus.write_byte(address, self.reg.b)
            }
            0xDD71 => {
                // LD (IX+d),C
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                bus.write_byte(address, self.reg.c)
            }
            0xDD72 => {
                // LD (IX+d),D
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                bus.write_byte(address, self.reg.d)
            }
            0xDD73 => {
                // LD (IX+d),E
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                bus.write_byte(address, self.reg.e)
            }
            0xDD74 => {
                // LD (IX+d),H
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                bus.write_byte(address, self.reg.h)
            }
            0xDD75 => {
                // LD (IX+d),L
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                bus.write_byte(address, self.reg.l)
            }
            0xDD77 => {
                // LD (IX+d),A
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                bus.write_byte(address, self.reg.a)
            }

            // LD (IY+d),r
            0xFD70 => {
                // LD (IY+d),B
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                bus.write_byte(address, self.reg.b)
            }
            0xFD71 => {
                // LD (IY+d),C
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                bus.write_byte(address, self.reg.c)
            }
            0xFD72 => {
                // LD (IY+d),D
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                bus.write_byte(address, self.reg.d)
            }
            0xFD73 => {
                // LD (IY+d),E
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                bus.write_byte(address, self.reg.e)
            }
            0xFD74 => {
                // LD (IY+d),H
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                bus.write_byte(address, self.reg.h)
            }
            0xFD75 => {
                // LD (IY+d),L
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                bus.write_byte(address, self.reg.l)
            }
            0xFD77 => {
                // LD (IY+d),A
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                bus.write_byte(address, self.reg.a)
            }

            // LD (IX+d),n
            0xDD36 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let data = bus.read_byte(self.reg.pc.wrapping_add(3));
                let address = self.ix_d(displacement);
                bus.write_byte(address, data)
            }

            // LD IX,nn
            0xDD21 => {
                self.reg.set_ix(bus.read_word(self.reg.pc.wrapping_add(2)));
            }

            // LD IY,nn
            0xFD21 => {
                self.reg.set_iy(bus.read_word(self.reg.pc.wrapping_add(2)));
            }

            // LD (IY+d),n
            0xFD36 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let data = bus.read_byte(self.reg.pc.wrapping_add(3));
                let address = self.iy_d(displacement);
                bus.write_byte(address, data)
            }

            // LD A,I
            0xED57 => {
                self.reg.a = self.reg.i;
                self.reg.flags.s = self.reg.i & 0x80 == 0x80;
                self.reg.flags.set_undocumented_from(self.reg.i);
                self.reg.flags.z = self.reg.i == 0;
                self.reg.flags.h = false;
                // The P/V flag mirrors IFF2.
                // Note: if an interrupt arrives at the same moment, P/V is forced to 0.
                self.reg.flags.p = if self.interrupt_pending_during_instruction() {
                    false
                } else {
                    self.iff2
                };
                self.reg.flags.n = false;
            }

            // LD A,R
            0xED5F => {
                self.reg.a = self.reg.r;
                self.reg.flags.s = self.reg.r & 0x80 == 0x80;
                self.reg.flags.set_undocumented_from(self.reg.r);
                self.reg.flags.z = self.reg.r == 0;
                self.reg.flags.h = false;
                self.reg.flags.p = if self.interrupt_pending_during_instruction() {
                    false
                } else {
                    self.iff2
                };
                self.reg.flags.n = false;
            }

            // LD I,A
            0xED47 => self.reg.i = self.reg.a,

            // LD R,A
            0xED4F => self.reg.r = self.reg.a,

            // 16-Bit Load Group
            // LD dd,(nn)
            0xED4B => {
                // LD BC,(nn)
                let addr = bus.read_word(self.reg.pc.wrapping_add(2));
                self.wz_after(addr);
                let d = bus.read_word(addr);
                self.reg.set_bc(d);
            }

            0xED5B => {
                // LD DE,(nn)
                let addr = bus.read_word(self.reg.pc.wrapping_add(2));
                self.wz_after(addr);
                let d = bus.read_word(addr);
                self.reg.set_de(d);
            }

            0xED6B => {
                // LD HL,(nn)
                let addr = bus.read_word(self.reg.pc.wrapping_add(2));
                self.wz_after(addr);
                let d = bus.read_word(addr);
                self.reg.set_hl(d);
            }

            0xED7B => {
                // LD SP,(nn)
                let addr = bus.read_word(self.reg.pc.wrapping_add(2));
                self.wz_after(addr);
                let d = bus.read_word(addr);
                self.reg.sp = d;
            }

            // LD IX,(nn)
            0xDD2A => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(2));
                self.wz_after(addr);
                let d = bus.read_word(addr);
                self.reg.set_ix(d);
            }

            // LD IY,(nn)
            0xFD2A => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(2));
                self.wz_after(addr);
                let d = bus.read_word(addr);
                self.reg.set_iy(d);
            }

            // LD (nn),dd
            0xED43 => {
                // LD (nn),BC
                let addr = bus.read_word(self.reg.pc.wrapping_add(2));
                self.wz_after(addr);
                bus.write_word(addr, self.reg.get_bc());
            }

            0xED53 => {
                // LD (nn),DE
                let addr = bus.read_word(self.reg.pc.wrapping_add(2));
                self.wz_after(addr);
                bus.write_word(addr, self.reg.get_de());
            }

            0xED63 => {
                // LD (nn),HL
                let addr = bus.read_word(self.reg.pc.wrapping_add(2));
                self.wz_after(addr);
                bus.write_word(addr, self.reg.get_hl());
            }

            0xED73 => {
                // LD (nn),SP
                let addr = bus.read_word(self.reg.pc.wrapping_add(2));
                self.wz_after(addr);
                bus.write_word(addr, self.reg.sp);
            }

            // LD (nn),IX
            0xDD22 => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(2));
                self.wz_after(addr);
                bus.write_word(addr, self.reg.get_ix());
            }

            // LD (nn),IY
            0xFD22 => {
                let addr = bus.read_word(self.reg.pc.wrapping_add(2));
                self.wz_after(addr);
                bus.write_word(addr, self.reg.get_iy());
            }

            // LD SP,IX
            0xDDF9 => self.reg.sp = self.reg.get_ix(),

            // LD SP,IY
            0xFDF9 => self.reg.sp = self.reg.get_iy(),

            // PUSH IX
            0xDDE5 => {
                self.reg.sp = self.reg.sp.wrapping_sub(2);
                bus.write_word(self.reg.sp, self.reg.get_ix());
            }

            // PUSH IY
            0xFDE5 => {
                self.reg.sp = self.reg.sp.wrapping_sub(2);
                bus.write_word(self.reg.sp, self.reg.get_iy());
            }

            // POP IX
            0xDDE1 => {
                self.reg.set_ix(bus.read_word(self.reg.sp));
                self.reg.sp = self.reg.sp.wrapping_add(2);
            }

            // POP IY
            0xFDE1 => {
                self.reg.set_iy(bus.read_word(self.reg.sp));
                self.reg.sp = self.reg.sp.wrapping_add(2);
            }

            // Exchange, Block Transfer, and Search Group
            // EX (SP),IX
            0xDDE3 => {
                let pointed_by_sp = bus.read_word(self.reg.sp);
                bus.write_word(self.reg.sp, self.reg.get_ix());
                self.reg.set_ix(pointed_by_sp);
                self.reg.wz = pointed_by_sp;
            }

            // EX (SP),IY
            0xFDE3 => {
                let pointed_by_sp = bus.read_word(self.reg.sp);
                bus.write_word(self.reg.sp, self.reg.get_iy());
                self.reg.set_iy(pointed_by_sp);
                self.reg.wz = pointed_by_sp;
            }

            // LDI
            0xEDA0 => {
                self.ldi(bus);
                self.reg.flags.h = false;
                let bc = self.reg.get_bc();
                self.reg.flags.p = bc != 0;
                self.reg.flags.n = false;
            }

            // Repeating instructions (LDIR, LDDR, CPIR, CPDR, INIR, INDR,
            // OTIR, OTDR)
            // -------------------------------------------------------------------------
            // The Z80 does NOT execute them in a single stretch: it performs
            // one iteration, and if the repeat condition still holds, it
            // steps PC back by two to replay the instruction. That's what
            // makes them interruptible between two iterations.
            //
            // Unrolling them entirely in a single call looks equivalent —
            // the reported cycle count is the same — but it freezes the
            // host machine for the whole duration: a 16 KB LDIR consumes
            // 344,000 cycles, more than four CPC frames, during which no
            // interrupt can be accepted. Any music or game logic clocked by
            // interrupts then falls behind.

            // LDIR
            0xEDB0 => {
                self.ldi(bus);
                let bc = self.reg.get_bc();
                self.reg.flags.h = false;
                self.reg.flags.p = bc != 0;
                self.reg.flags.n = false;
                cycles = self.repeat_block_wz(bc != 0);
            }

            // LDD
            0xEDA8 => {
                self.ldd(bus);
                self.reg.flags.h = false;
                let bc = self.reg.get_bc();
                self.reg.flags.p = bc != 0;
                self.reg.flags.n = false;
            }

            // LDDR
            0xEDB8 => {
                self.ldd(bus);
                let bc = self.reg.get_bc();
                self.reg.flags.h = false;
                self.reg.flags.p = bc != 0;
                self.reg.flags.n = false;
                cycles = self.repeat_block_wz(bc != 0);
            }

            // CPI
            0xEDA1 => self.cpi(bus),

            // CPIR
            0xEDB1 => {
                self.cpi(bus);
                // The repetition stops on a match (Z=1) or when BC reaches zero.
                let again = !self.reg.flags.z && self.reg.get_bc() != 0;
                cycles = self.repeat_block_wz(again);
            }

            // CPD
            0xEDA9 => self.cpd(bus),

            // CPDR
            0xEDB9 => {
                self.cpd(bus);
                // The repetition stops on a match (Z=1) or when BC reaches zero.
                let again = !self.reg.flags.z && self.reg.get_bc() != 0;
                cycles = self.repeat_block_wz(again);
            }

            // 8-Bit Arithmetic Group
            // ADD A,(IX+d)
            0xDD86 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                let d = bus.read_byte(address);
                self.add(d);
            }

            // ADD A,(IY+d)
            0xFD86 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                let d = bus.read_byte(address);
                self.add(d);
            }

            // ADC A,(IX+d)
            0xDD8E => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                let d = bus.read_byte(address);
                self.adc(d);
            }

            // ADC A,(IY+d)
            0xFD8E => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                let d = bus.read_byte(address);
                self.adc(d);
            }

            // SUB (IX+d)
            0xDD96 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                let d = bus.read_byte(address);
                self.sub(d);
            }

            // SUB (IY+d)
            0xFD96 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                let d = bus.read_byte(address);
                self.sub(d);
            }

            // SBC (IX+d)
            0xDD9E => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                let d = bus.read_byte(address);
                self.sbc(d);
            }

            // SBC (IY+d)
            0xFD9E => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                let d = bus.read_byte(address);
                self.sbc(d);
            }

            // AND (IX+d)
            0xDDA6 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                let d = bus.read_byte(address);
                self.and(d);
            }

            // AND (IY+d)
            0xFDA6 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                let d = bus.read_byte(address);
                self.and(d);
            }

            // OR (IX+d)
            0xDDB6 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                let d = bus.read_byte(address);
                self.or(d);
            }

            // OR (IY+d)
            0xFDB6 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                let d = bus.read_byte(address);
                self.or(d);
            }

            // XOR (IX+d)
            0xDDAE => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                let d = bus.read_byte(address);
                self.xor(d);
            }

            // XOR (IY+d)
            0xFDAE => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                let d = bus.read_byte(address);
                self.xor(d);
            }

            // CP (IX+d)
            0xDDBE => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                let d = bus.read_byte(address);
                self.cp(d);
            }

            // CP (IY+d)
            0xFDBE => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                let d = bus.read_byte(address);
                self.cp(d);
            }

            // INC (IX+d)
            0xDD34 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                let m = address;
                let d = bus.read_byte(m);
                let r = self.inc(d);
                bus.write_byte(m, r);
            }

            // INC (IY+d)
            0xFD34 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                let m = address;
                let d = bus.read_byte(m);
                let r = self.inc(d);
                bus.write_byte(m, r);
            }

            // DEC (IX+d)
            0xDD35 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.ix_d(displacement);
                let m = address;
                let d = bus.read_byte(m);
                let r = self.dec(d);
                bus.write_byte(m, r);
            }

            // DEC (IY+d)
            0xFD35 => {
                let displacement = bus.read_byte(self.reg.pc.wrapping_add(2));
                let address = self.iy_d(displacement);
                let m = address;
                let d = bus.read_byte(m);
                let r = self.dec(d);
                bus.write_byte(m, r);
            }

            // General-Purpose Arithmetic and CPU Control Groups
            // NEG. The Z80 only decodes three bits of this field: the other
            // seven combinations (0x4C, 0x54...) are the SAME NEG,
            // undocumented but very real. Leaving them to the "ED table gap"
            // fallback would turn them into silent NOPs.
            0xED44 | 0xED4C | 0xED54 | 0xED5C | 0xED64 | 0xED6C | 0xED74 | 0xED7C => {
                self.neg();
            }

            // RETI
            0xED4D => {
                self.iff1 = self.iff2;
                self.call_stack_pop(bus);
            }

            // RETN, and its six undocumented duplicates (same reason as NEG
            // above).
            0xED45 | 0xED55 | 0xED5D | 0xED65 | 0xED6D | 0xED75 | 0xED7D => {
                self.iff1 = self.iff2;
                self.call_stack_pop(bus);
            }

            // Interrput modes
            0xED46 => self.im = 0,
            0xED56 => self.im = 1,
            0xED5E => self.im = 2,

            //16-Bit Arithmetic Group
            // ADC HL,ss
            0xED4A => {
                // ADC HL,BC
                let reg = self.reg.get_bc();
                self.adc_16(reg);
            }
            0xED5A => {
                // ADC HL,DE
                let reg = self.reg.get_de();
                self.adc_16(reg);
            }
            0xED6A => {
                // ADC HL,HL
                let reg = self.reg.get_hl();
                self.adc_16(reg);
            }
            0xED7A => {
                // ADC HL,SP
                let reg = self.reg.sp;
                self.adc_16(reg);
            }

            // SBC HL,ss
            0xED42 => {
                // SBC HL,BC
                let reg = self.reg.get_bc();
                self.sbc_16(reg);
            }
            0xED52 => {
                // SBC HL,DE
                let reg = self.reg.get_de();
                self.sbc_16(reg);
            }
            0xED62 => {
                // SBC HL,HL
                let reg = self.reg.get_hl();
                self.sbc_16(reg);
            }
            0xED72 => {
                // SBC HL,SP
                let reg = self.reg.sp;
                self.sbc_16(reg);
            }

            // ADD IX,pp
            0xDD09 => {
                // ADD IX,BC
                let reg = self.reg.get_bc();
                let r = self.add_16(self.reg.get_ix(), reg);
                self.reg.set_ix(r);
            }
            0xDD19 => {
                // ADD IX,DE
                let reg = self.reg.get_de();
                let r = self.add_16(self.reg.get_ix(), reg);
                self.reg.set_ix(r);
            }
            0xDD29 => {
                // ADD IX,IX
                let reg = self.reg.get_ix();
                let r = self.add_16(self.reg.get_ix(), reg);
                self.reg.set_ix(r);
            }
            0xDD39 => {
                // ADD IX,SP
                let reg = self.reg.sp;
                let r = self.add_16(self.reg.get_ix(), reg);
                self.reg.set_ix(r);
            }

            // ADD IY,pp
            0xFD09 => {
                // ADD IY,BC
                let reg = self.reg.get_bc();
                let r = self.add_16(self.reg.get_iy(), reg);
                self.reg.set_iy(r);
            }

            0xFD19 => {
                // ADD IY,DE
                let reg = self.reg.get_de();
                let r = self.add_16(self.reg.get_iy(), reg);
                self.reg.set_iy(r);
            }

            0xFD29 => {
                // ADD IY,IY
                let reg = self.reg.get_iy();
                let r = self.add_16(self.reg.get_iy(), reg);
                self.reg.set_iy(r);
            }

            0xFD39 => {
                // ADD IY,SP
                let reg = self.reg.sp;
                let r = self.add_16(self.reg.get_iy(), reg);
                self.reg.set_iy(r);
            }

            0xDD23 => {
                // INC IX
                let r = self.reg.get_ix().wrapping_add(1);
                self.reg.set_ix(r);
            }

            0xFD23 => {
                // INC IY
                let r = self.reg.get_iy().wrapping_add(1);
                self.reg.set_iy(r);
            }

            0xDD2B => {
                // DEC IX
                let r = self.reg.get_ix().wrapping_sub(1);
                self.reg.set_ix(r);
            }

            0xFD2B => {
                // DEC IY
                let r = self.reg.get_iy().wrapping_sub(1);
                self.reg.set_iy(r);
            }

            // Rotate and Shift Group
            // RLC r
            0xCB00 => {
                // RLC B
                let r = self.rlc(self.reg.b);
                self.reg.b = r;
            }

            0xCB01 => {
                // RLC C
                let r = self.rlc(self.reg.c);
                self.reg.c = r;
            }

            0xCB02 => {
                // RLC D
                let r = self.rlc(self.reg.d);
                self.reg.d = r;
            }

            0xCB03 => {
                // RLC E
                let r = self.rlc(self.reg.e);
                self.reg.e = r;
            }

            0xCB04 => {
                // RLC H
                let r = self.rlc(self.reg.h);
                self.reg.h = r;
            }

            0xCB05 => {
                // RLC L
                let r = self.rlc(self.reg.l);
                self.reg.l = r;
            }

            0xCB06 => {
                // RLC (HL)
                let addr = self.reg.get_hl();
                let r = self.rlc(bus.read_byte(addr));
                bus.write_byte(addr, r);
            }

            0xCB07 => {
                // RLC A
                let r = self.rlc(self.reg.a);
                self.reg.a = r;
            }

            // RL r
            0xCB10 => {
                // RL B
                let r = self.rl(self.reg.b);
                self.reg.b = r;
            }

            0xCB11 => {
                // RL C
                let r = self.rl(self.reg.c);
                self.reg.c = r;
            }

            0xCB12 => {
                // RL D
                let r = self.rl(self.reg.d);
                self.reg.d = r;
            }

            0xCB13 => {
                // RL E
                let r = self.rl(self.reg.e);
                self.reg.e = r;
            }

            0xCB14 => {
                // RL H
                let r = self.rl(self.reg.h);
                self.reg.h = r;
            }

            0xCB15 => {
                // RL L
                let r = self.rl(self.reg.l);
                self.reg.l = r;
            }

            0xCB16 => {
                // RL (HL)
                let addr = self.reg.get_hl();
                let r = self.rl(bus.read_byte(addr));
                bus.write_byte(addr, r);
            }

            0xCB17 => {
                // RL A
                let r = self.rl(self.reg.a);
                self.reg.a = r;
            }

            // RRC r
            0xCB08 => {
                // RRC B
                let r = self.rrc(self.reg.b);
                self.reg.b = r;
            }

            0xCB09 => {
                // RRC C
                let r = self.rrc(self.reg.c);
                self.reg.c = r;
            }

            0xCB0A => {
                // RRC D
                let r = self.rrc(self.reg.d);
                self.reg.d = r;
            }

            0xCB0B => {
                // RRC E
                let r = self.rrc(self.reg.e);
                self.reg.e = r;
            }

            0xCB0C => {
                // RRC H
                let r = self.rrc(self.reg.h);
                self.reg.h = r;
            }

            0xCB0D => {
                // RRC L
                let r = self.rrc(self.reg.l);
                self.reg.l = r;
            }

            0xCB0E => {
                // RR (HL)
                let addr = self.reg.get_hl();
                let r = self.rrc(bus.read_byte(addr));
                bus.write_byte(addr, r);
            }

            0xCB0F => {
                // RR A
                let r = self.rrc(self.reg.a);
                self.reg.a = r;
            }

            // RR r
            0xCB18 => {
                // RR B
                let r = self.rr(self.reg.b);
                self.reg.b = r;
            }

            0xCB19 => {
                // RR C
                let r = self.rr(self.reg.c);
                self.reg.c = r;
            }

            0xCB1A => {
                // RR D
                let r = self.rr(self.reg.d);
                self.reg.d = r;
            }

            0xCB1B => {
                // RR E
                let r = self.rr(self.reg.e);
                self.reg.e = r;
            }

            0xCB1C => {
                // RR H
                let r = self.rr(self.reg.h);
                self.reg.h = r;
            }

            0xCB1D => {
                // RR L
                let r = self.rr(self.reg.l);
                self.reg.l = r;
            }

            0xCB1E => {
                // RR (HL)
                let addr = self.reg.get_hl();
                let r = self.rr(bus.read_byte(addr));
                bus.write_byte(addr, r);
            }

            0xCB1F => {
                // RR A
                let r = self.rr(self.reg.a);
                self.reg.a = r;
            }

            // SLA r
            0xCB20 => {
                // SLA B
                let r = self.sla(self.reg.b);
                self.reg.b = r;
            }

            0xCB21 => {
                // SLA C
                let r = self.sla(self.reg.c);
                self.reg.c = r;
            }

            0xCB22 => {
                // SLA D
                let r = self.sla(self.reg.d);
                self.reg.d = r;
            }

            0xCB23 => {
                // SLA E
                let r = self.sla(self.reg.e);
                self.reg.e = r;
            }

            0xCB24 => {
                // SLA H
                let r = self.sla(self.reg.h);
                self.reg.h = r;
            }

            0xCB25 => {
                // SLA L
                let r = self.sla(self.reg.l);
                self.reg.l = r;
            }

            0xCB26 => {
                // SLA (HL)
                let addr = self.reg.get_hl();
                let r = self.sla(bus.read_byte(addr));
                bus.write_byte(addr, r);
            }

            0xCB27 => {
                // SLA A
                let r = self.sla(self.reg.a);
                self.reg.a = r;
            }

            // SRA r
            0xCB28 => {
                // SRA B
                let r = self.sra(self.reg.b);
                self.reg.b = r;
            }

            0xCB29 => {
                // SRA C
                let r = self.sra(self.reg.c);
                self.reg.c = r;
            }

            0xCB2A => {
                // SRA D
                let r = self.sra(self.reg.d);
                self.reg.d = r;
            }

            0xCB2B => {
                // SRA E
                let r = self.sra(self.reg.e);
                self.reg.e = r;
            }

            0xCB2C => {
                // SRA H
                let r = self.sra(self.reg.h);
                self.reg.h = r;
            }

            0xCB2D => {
                // SRA L
                let r = self.sra(self.reg.l);
                self.reg.l = r;
            }

            0xCB2E => {
                // SRA (HL)
                let addr = self.reg.get_hl();
                let r = self.sra(bus.read_byte(addr));
                bus.write_byte(addr, r);
            }

            0xCB2F => {
                // SRA A
                let r = self.sra(self.reg.a);
                self.reg.a = r;
            }

            // SRL r
            0xCB38 => {
                // SRL B
                let r = self.srl(self.reg.b);
                self.reg.b = r;
            }

            0xCB39 => {
                // SRL C
                let r = self.srl(self.reg.c);
                self.reg.c = r;
            }

            0xCB3A => {
                // SRL D
                let r = self.srl(self.reg.d);
                self.reg.d = r;
            }

            0xCB3B => {
                // SRL E
                let r = self.srl(self.reg.e);
                self.reg.e = r;
            }

            0xCB3C => {
                // SRL H
                let r = self.srl(self.reg.h);
                self.reg.h = r;
            }

            0xCB3D => {
                // SRL L
                let r = self.srl(self.reg.l);
                self.reg.l = r;
            }

            0xCB3E => {
                // SRL (HL)
                let addr = self.reg.get_hl();
                let r = self.srl(bus.read_byte(addr));
                bus.write_byte(addr, r);
            }

            0xCB3F => {
                // SRL A
                let r = self.srl(self.reg.a);
                self.reg.a = r;
            }

            // RLD
            0xED6F => {
                let hl_contents = bus.read_byte(self.reg.get_hl());
                let a_contents = self.reg.a;

                let r = (self.reg.a & 0xF0) | ((hl_contents & 0xF0) >> 4);
                self.reg.a = r;
                bus.write_byte(self.reg.get_hl(), (hl_contents << 4) | (a_contents & 0x0F));
                self.reg.flags.s = r & 0x80 == 0x80;
                self.reg.flags.set_undocumented_from(r);
                self.reg.flags.z = r == 0x00;
                self.reg.flags.h = false;
                self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
                self.reg.flags.n = false;
            }

            // RRD
            0xED67 => {
                let hl_contents = bus.read_byte(self.reg.get_hl());
                let a_contents = self.reg.a;

                let r = (self.reg.a & 0xF0) | (hl_contents & 0x0F);
                self.reg.a = r;
                bus.write_byte(
                    self.reg.get_hl(),
                    ((a_contents & 0x0F) << 4) | ((hl_contents & 0xF0) >> 4),
                );
                self.reg.flags.s = r & 0x80 == 0x80;
                self.reg.flags.set_undocumented_from(r);
                self.reg.flags.z = r == 0x00;
                self.reg.flags.h = false;
                self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
                self.reg.flags.n = false;
            }

            // Bit Set, Reset, and Test Group
            // BIT b,r
            0xCB40..=0xCB7F => self.bit(bus, bus.read_byte(self.reg.pc.wrapping_add(1))),

            // SET b,r
            0xCBC0..=0xCBFF => self.set(bus, bus.read_byte(self.reg.pc.wrapping_add(1))),

            // RES b,r
            0xCB80..=0xCBBF => self.reset(bus, bus.read_byte(self.reg.pc.wrapping_add(1))),

            // Jump group
            // JP (IX)
            0xDDE9 => {
                self.reg.pc = self.reg.get_ix();
            }

            // JP (IY)
            0xFDE9 => {
                self.reg.pc = self.reg.get_iy();
            }

            // Undocumented instructions
            // ADD A,IXH
            0xDD84 => {
                let n = self.reg.ixh;
                self.add(n);
            }

            // ADD A,IXL
            0xDD85 => {
                let n = self.reg.ixl;
                self.add(n);
            }

            // ADD A,IYH
            0xFD84 => {
                let n = self.reg.iyh;
                self.add(n);
            }

            // ADD A,IYL
            0xFD85 => {
                let n = self.reg.iyl;
                self.add(n);
            }

            // ADC A,IXH
            0xDD8C => {
                let n = self.reg.ixh;
                self.adc(n);
            }

            // ADC A,IXL
            0xDD8D => {
                let n = self.reg.ixl;
                self.adc(n);
            }

            // ADC A,IYH
            0xFD8C => {
                let n = self.reg.iyh;
                self.adc(n);
            }

            // ADC A,IYL
            0xFD8D => {
                let n = self.reg.iyl;
                self.adc(n);
            }

            // SUB IXH
            0xDD94 => {
                let n = self.reg.ixh;
                self.sub(n);
            }

            // SUB IXL
            0xDD95 => {
                let n = self.reg.ixl;
                self.sub(n);
            }

            // SUB IYH
            0xFD94 => {
                let n = self.reg.iyh;
                self.sub(n);
            }

            // SUB IYL
            0xFD95 => {
                let n = self.reg.iyl;
                self.sub(n);
            }

            // SBC A,IXH
            0xDD9C => {
                let n = self.reg.ixh;
                self.sbc(n);
            }

            // SBC A,IXL
            0xDD9D => {
                let n = self.reg.ixl;
                self.sbc(n);
            }

            // SBC A,IYH
            0xFD9C => {
                let n = self.reg.iyh;
                self.sbc(n);
            }

            // SBC A,IYL
            0xFD9D => {
                let n = self.reg.iyl;
                self.sbc(n);
            }

            // AND IXH
            0xDDA4 => {
                let n = self.reg.ixh;
                self.and(n);
            }

            // AND IXL
            0xDDA5 => {
                let n = self.reg.ixl;
                self.and(n);
            }

            // AND IYH
            0xFDA4 => {
                let n = self.reg.iyh;
                self.and(n);
            }

            // AND IYL
            0xFDA5 => {
                let n = self.reg.iyl;
                self.and(n);
            }

            // OR IXH
            0xDDB4 => {
                let n = self.reg.ixh;
                self.or(n);
            }

            // OR IXL
            0xDDB5 => {
                let n = self.reg.ixl;
                self.or(n);
            }

            // OR IYH
            0xFDB4 => {
                let n = self.reg.iyh;
                self.or(n);
            }

            // OR IYL
            0xFDB5 => {
                let n = self.reg.iyl;
                self.or(n);
            }

            // XOR IXH
            0xDDAC => {
                let n = self.reg.ixh;
                self.xor(n);
            }

            // XOR IXL
            0xDDAD => {
                let n = self.reg.ixl;
                self.xor(n);
            }

            // XOR IYH
            0xFDAC => {
                let n = self.reg.iyh;
                self.xor(n);
            }

            // XOR IYL
            0xFDAD => {
                let n = self.reg.iyl;
                self.xor(n);
            }

            // CP IXH
            0xDDBC => {
                let n = self.reg.ixh;
                self.cp(n);
            }

            // CP IXL
            0xDDBD => {
                let n = self.reg.ixl;
                self.cp(n);
            }

            // CP IYH
            0xFDBC => {
                let n = self.reg.iyh;
                self.cp(n);
            }

            // CP IYL
            0xFDBD => {
                let n = self.reg.iyl;
                self.cp(n);
            }

            // INC IXH
            0xDD24 => {
                let n = self.reg.ixh;
                let r = self.inc(n);
                self.reg.ixh = r;
            }

            // DEC IXH
            0xDD25 => {
                let n = self.reg.ixh;
                let r = self.dec(n);
                self.reg.ixh = r;
            }

            // INC IXL
            0xDD2C => {
                let n = self.reg.ixl;
                let r = self.inc(n);
                self.reg.ixl = r;
            }

            // DEC IXL
            0xDD2D => {
                let n = self.reg.ixl;
                let r = self.dec(n);
                self.reg.ixl = r;
            }

            // INC IYH
            0xFD24 => {
                let n = self.reg.iyh;
                let r = self.inc(n);
                self.reg.iyh = r;
            }

            // DEC IYH
            0xFD25 => {
                let n = self.reg.iyh;
                let r = self.dec(n);
                self.reg.iyh = r;
            }

            // INC IYL
            0xFD2C => {
                let n = self.reg.iyl;
                let r = self.inc(n);
                self.reg.iyl = r;
            }
            // DEC IYL
            0xFD2D => {
                let n = self.reg.iyl;
                let r = self.dec(n);
                self.reg.iyl = r;
            }

            // LD IXH,n
            0xDD26 => {
                let n = bus.read_byte(self.reg.pc.wrapping_add(2));
                self.reg.ixh = n;
            }

            // LD IYH,n
            0xFD26 => {
                let n = bus.read_byte(self.reg.pc.wrapping_add(2));
                self.reg.iyh = n;
            }

            // LD IXL,n
            0xDD2E => {
                let n = bus.read_byte(self.reg.pc.wrapping_add(2));
                self.reg.ixl = n;
            }

            // LD IYL,n
            0xFD2E => {
                let n = bus.read_byte(self.reg.pc.wrapping_add(2));
                self.reg.iyl = n;
            }

            // LD B,B
            0xDD40 | 0xFD40 => {}

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
            0xDD49 | 0xFD49 => {}

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
            0xDD52 | 0xFD52 => {}

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
            0xDD5B | 0xFD5B => {}

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
            0xDD64 => {}

            // LD IYH,IYH
            0xFD64 => {}

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
            0xDD6D => {}

            // LD IYL,IYL
            0xFD6D => {}

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
            0xDD7F | 0xFD7F => {}

            // SLL B
            0xCB30 => {
                let r = self.sll(self.reg.b);
                self.reg.b = r;
            }

            // SLL C
            0xCB31 => {
                let r = self.sll(self.reg.c);
                self.reg.c = r;
            }

            // SLL D
            0xCB32 => {
                let r = self.sll(self.reg.d);
                self.reg.d = r;
            }

            // SLL E
            0xCB33 => {
                let r = self.sll(self.reg.e);
                self.reg.e = r;
            }

            // SLL H
            0xCB34 => {
                let r = self.sll(self.reg.h);
                self.reg.h = r;
            }

            // SLL L
            0xCB35 => {
                let r = self.sll(self.reg.l);
                self.reg.l = r;
            }

            // SLL (HL)
            0xCB36 => {
                let addr = self.reg.get_hl();
                let r = self.sll(bus.read_byte(addr));
                bus.write_byte(addr, r);
            }

            // SLL A
            0xCB37 => {
                let r = self.sll(self.reg.a);
                self.reg.a = r;
            }

            // =========================================================================
            // INPUT / OUTPUT (I/O) INSTRUCTION GROUP - OPCODES 0xED
            // =========================================================================

            // IN r, (C) - Reads port BC and writes into the specified register.
            // Modifies flags S, Z, H (reset), P (parity), N (reset).
            0xED40 | 0xED48 | 0xED50 | 0xED58 | 0xED60 | 0xED68 | 0xED78 => {
                let port = self.reg.get_bc();
                let data = bus.read_io(port);

                // Routes to the right destination register depending on the opcode
                match opcode {
                    0xED40 => self.reg.b = data, // IN B, (C)
                    0xED48 => self.reg.c = data, // IN C, (C)
                    0xED50 => self.reg.d = data, // IN D, (C)
                    0xED58 => self.reg.e = data, // IN E, (C)
                    0xED60 => self.reg.h = data, // IN H, (C)
                    0xED68 => self.reg.l = data, // IN L, (C)
                    0xED78 => self.reg.a = data, // IN A, (C)
                    _ => {}
                }

                // Update flags
                self.reg.flags.s = (data & 0x80) != 0;
                self.reg.flags.set_undocumented_from(data);
                self.reg.flags.z = data == 0;
                self.reg.flags.h = false;
                self.reg.flags.p = data.count_ones() & 0x01 == 0x00; // Parity
                self.reg.flags.n = false;
                self.wz_after(port);
            }

            // IN F, (C) - Opcode 0xED70 (Undocumented): only affects flags
            0xED70 => {
                let port = self.reg.get_bc();
                let data = bus.read_io(port);
                self.reg.flags.s = (data & 0x80) != 0;
                self.reg.flags.set_undocumented_from(data);
                self.reg.flags.z = data == 0;
                self.reg.flags.h = false;
                self.reg.flags.p = data.count_ones() & 0x01 == 0x00;
                self.reg.flags.n = false;
                self.wz_after(port);
            }

            // OUT (C), r - Writes the specified register's value to port BC.
            0xED41 | 0xED49 | 0xED51 | 0xED59 | 0xED61 | 0xED69 | 0xED79 => {
                let port = self.reg.get_bc();
                let data = match opcode {
                    0xED41 => self.reg.b, // OUT (C), B
                    0xED49 => self.reg.c, // OUT (C), C
                    0xED51 => self.reg.d, // OUT (C), D
                    0xED59 => self.reg.e, // OUT (C), E
                    0xED61 => self.reg.h, // OUT (C), H
                    0xED69 => self.reg.l, // OUT (C), L
                    0xED79 => self.reg.a, // OUT (C), A
                    _ => 0,
                };
                bus.write_io(port, data);
                self.wz_after(port);
            }

            // OUT (C), 0 - Opcode 0xED71 (Undocumented): writes a null byte to port BC.
            0xED71 => {
                let port = self.reg.get_bc();
                bus.write_io(port, 0);
                self.wz_after(port);
            }

            // -------------------------------------------------------------------------
            // Block I/O transfers (opcodes INI, INIR, IND, INDR, OUTI, OTIR, OUTD, OTDR)
            // -------------------------------------------------------------------------
            // Watch for the asymmetry between the two families: OUTPUT
            // instructions decrement B BEFORE the port access, so it's B-1
            // that's presented on A8-A15, whereas INPUT instructions
            // decrement it AFTER and present B unchanged. That's why "INC B"
            // is always seen right before OUTI in period code.

            // INI (0xEDA2): reads from port BC, writes to (HL), increments HL, decrements B
            0xEDA2 => {
                let port = self.reg.get_bc();
                let data = bus.read_io(port);
                bus.write_byte(self.reg.get_hl(), data);
                self.reg.set_hl(self.reg.get_hl().wrapping_add(1));
                self.reg.b = self.reg.b.wrapping_sub(1);

                self.block_io_flags(data, self.reg.c.wrapping_add(1));
                self.reg.wz = self.reg.get_bc().wrapping_add(1);
            }

            // INIR (0xEDB2): INI repeated until B reaches 0
            0xEDB2 => {
                let port = self.reg.get_bc();
                let data = bus.read_io(port);
                bus.write_byte(self.reg.get_hl(), data);
                self.reg.set_hl(self.reg.get_hl().wrapping_add(1));
                self.reg.b = self.reg.b.wrapping_sub(1);

                self.block_io_flags(data, self.reg.c.wrapping_add(1));
                self.reg.wz = self.reg.get_bc().wrapping_add(1);
                cycles = self.repeat_block(self.reg.b != 0);
            }

            // IND (0xEDAA): reads from port BC, writes to (HL), decrements HL, decrements B
            0xEDAA => {
                let port = self.reg.get_bc();
                let data = bus.read_io(port);
                bus.write_byte(self.reg.get_hl(), data);
                self.reg.set_hl(self.reg.get_hl().wrapping_sub(1));
                self.reg.b = self.reg.b.wrapping_sub(1);

                self.block_io_flags(data, self.reg.c.wrapping_sub(1));
                self.reg.wz = self.reg.get_bc().wrapping_sub(1);
            }

            // INDR (0xEDBA): IND repeated until B reaches 0
            0xEDBA => {
                let port = self.reg.get_bc();
                let data = bus.read_io(port);
                bus.write_byte(self.reg.get_hl(), data);
                self.reg.set_hl(self.reg.get_hl().wrapping_sub(1));
                self.reg.b = self.reg.b.wrapping_sub(1);

                self.block_io_flags(data, self.reg.c.wrapping_sub(1));
                self.reg.wz = self.reg.get_bc().wrapping_sub(1);
                cycles = self.repeat_block(self.reg.b != 0);
            }

            // OUTI (0xEDA3): reads from (HL), decrements B, writes to port BC, increments HL
            0xEDA3 => {
                let data = bus.read_byte(self.reg.get_hl());
                self.reg.b = self.reg.b.wrapping_sub(1);
                let port = self.reg.get_bc();
                bus.write_io(port, data);
                self.reg.set_hl(self.reg.get_hl().wrapping_add(1));

                self.block_io_flags(data, self.reg.l);
                self.reg.wz = self.reg.get_bc().wrapping_add(1);
            }

            // OTIR (0xEDB3): OUTI repeated until B reaches 0
            0xEDB3 => {
                let data = bus.read_byte(self.reg.get_hl());
                self.reg.b = self.reg.b.wrapping_sub(1);
                let port = self.reg.get_bc();
                bus.write_io(port, data);
                self.reg.set_hl(self.reg.get_hl().wrapping_add(1));

                self.block_io_flags(data, self.reg.l);
                self.reg.wz = self.reg.get_bc().wrapping_add(1);
                cycles = self.repeat_block(self.reg.b != 0);
            }

            // OUTD (0xEDAB): reads from (HL), decrements B, writes to port BC, decrements HL
            0xEDAB => {
                let data = bus.read_byte(self.reg.get_hl());
                self.reg.b = self.reg.b.wrapping_sub(1);
                let port = self.reg.get_bc();
                bus.write_io(port, data);
                self.reg.set_hl(self.reg.get_hl().wrapping_sub(1));

                self.block_io_flags(data, self.reg.l);
                self.reg.wz = self.reg.get_bc().wrapping_sub(1);
            }

            // OTDR (0xEDBB): OUTD repeated until B reaches 0
            0xEDBB => {
                let data = bus.read_byte(self.reg.get_hl());
                self.reg.b = self.reg.b.wrapping_sub(1);
                let port = self.reg.get_bc();
                bus.write_io(port, data);
                self.reg.set_hl(self.reg.get_hl().wrapping_sub(1));

                self.block_io_flags(data, self.reg.l);
                self.reg.wz = self.reg.get_bc().wrapping_sub(1);
                cycles = self.repeat_block(self.reg.b != 0);
            }

            _ => match (opcode >> 8) as u8 {
                // A DD/FD prefix in front of an instruction that touches
                // neither HL nor (HL) has no effect: the Z80 passes through
                // it in four cycles and executes the instruction as is.
                // That's the case for 145 DD opcodes and 149 FD opcodes,
                // which aren't missing, just moot.
                0xDD | 0xFD => {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                    return 4;
                }
                // The gaps in the ED table are not instructions: the
                // processor passes through them doing nothing, in eight cycles.
                0xED => cycles = 8,
                _ => self.record_unimplemented(bus, 2),
            },
        }

        match opcode {
            0xDDE9 | 0xFDE9 | 0xED4D | 0xED45 | 0xED55 | 0xED5D | 0xED65 | 0xED6D | 0xED75
            | 0xED7D => {}
            0xDD46 | 0xFD46 | 0xDD4E | 0xFD4E | 0xDD56 | 0xFD56 | 0xDD5E | 0xFD5E | 0xDD66
            | 0xFD66 | 0xDD6E | 0xFD6E | 0xDD7E | 0xFD7E | 0xDD70 | 0xDD71 | 0xDD72 | 0xDD73
            | 0xDD74 | 0xDD75 | 0xDD77 | 0xFD70 | 0xFD71 | 0xFD72 | 0xFD73 | 0xFD74 | 0xFD75
            | 0xFD77 | 0xDD86 | 0xFD86 | 0xDD8E | 0xFD8E | 0xDD96 | 0xFD96 | 0xDD9E | 0xFD9E
            | 0xDDA6 | 0xFDA6 | 0xDDB6 | 0xFDB6 | 0xDDAE | 0xFDAE | 0xDDBE | 0xFDBE | 0xDD34
            | 0xFD34 | 0xDD35 | 0xFD35 | 0xDD26 | 0xDD2E | 0xFD26 | 0xFD2E => {
                self.reg.pc = self.reg.pc.wrapping_add(3)
            }
            0xDD36 | 0xFD36 | 0xDD21 | 0xFD21 | 0xED4B | 0xED5B | 0xED6B | 0xED7B | 0xDD2A
            | 0xFD2A | 0xED43 | 0xED53 | 0xED63 | 0xED73 | 0xDD22 | 0xFD22 | 0xDDCB | 0xFDCB => {
                self.reg.pc = self.reg.pc.wrapping_add(4)
            }
            _ => self.reg.pc = self.reg.pc.wrapping_add(2),
        }

        cycles
    }

    // DDCB FDCB
    /// DD CB / FD CB instructions: bit operations on an indexed byte.
    ///
    /// Their format is perfectly regular — prefix, CB, displacement, then an
    /// opcode whose fields designate the operation and a register — which is
    /// what makes decoding them possible instead of enumerating them:
    ///
    /// ```text
    ///   7 6 5 4 3 2 1 0
    ///   x x y y y z z z
    /// ```
    ///
    /// The operation always acts on the targeted memory cell. When z doesn't
    /// designate that cell (z != 6), the result is ALSO copied into register
    /// z: an undocumented form, but a real one in silicon, and one that real
    /// programs use.
    fn execute_4bytes<B: Bus>(&mut self, bus: &mut B) -> u32 {
        let prefix = bus.read_byte(self.reg.pc);
        let displacement = bus.read_byte(self.reg.pc.wrapping_add(2)) as i8;
        let op = bus.read_byte(self.reg.pc.wrapping_add(3));
        self.reg.pc = self.reg.pc.wrapping_add(4);

        let address = if prefix == 0xDD {
            self.ix_d(displacement as u8)
        } else {
            self.iy_d(displacement as u8)
        };
        let value = bus.read_byte(address);
        let (x, y, z) = (op >> 6, ((op >> 3) & 7) as usize, op & 7);

        let result = match x {
            0 => match y {
                0 => self.rlc(value),
                1 => self.rrc(value),
                2 => self.rl(value),
                3 => self.rr(value),
                4 => self.sla(value),
                5 => self.sra(value),
                6 => self.sll(value),
                _ => self.srl(value),
            },
            1 => {
                // BIT only touches flags: neither memory nor a register,
                // and the z field has no effect.
                let r = bit::get(value, y);
                self.reg.flags.z = !r;
                self.reg.flags.h = true;
                self.reg.flags.n = false;
                self.reg.flags.s = r && y == 7;
                self.reg.flags.p = !r;
                // Like `BIT b,(HL)`, the indexed form takes its two
                // undocumented flags from MEMPTR's high byte — which
                // `ix_d`/`iy_d` just loaded with the targeted address.
                self.reg
                    .flags
                    .set_undocumented_from((self.reg.wz >> 8) as u8);
                return 20;
            }
            2 => bit::reset(value, y),
            _ => bit::set(value, y),
        };

        bus.write_byte(address, result);
        if z != 6 {
            self.set_register(z, result);
        }
        23
    }

    /// Advances register R by one M1 cycle (opcode fetch).
    ///
    /// The Z80 increments it on every opcode byte read from memory —
    /// including CB/ED/DD/FD prefix bytes, which are themselves M1 cycles.
    /// Only the low 7 bits count: bit 7 is only changed by an explicit
    /// write (LD R,A), never by the counting.
    fn bump_r(&mut self) {
        self.reg.r = (self.reg.r & 0x80) | (self.reg.r.wrapping_add(1) & 0x7F);
    }

    /// Records the current instruction as unhandled, without executing
    /// anything.
    ///
    /// The processor moves on to the next one: that's the least destructive
    /// behavior, and the host has what it needs to know what happened.
    fn record_unimplemented<B: Bus>(&mut self, bus: &B, len: u8) {
        let address = self.reg.pc;
        let mut bytes = [0u8; 4];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = bus.read_byte(address.wrapping_add(i as u16));
        }
        self.unimplemented = Some(Unimplemented {
            address,
            bytes,
            len,
        });
        self.unimplemented_count = self.unimplemented_count.saturating_add(1);
    }

    /// Stores a value into the 8-bit register designated by a y or z field.
    /// Code 6 designates a memory access, which callers handle themselves.
    fn set_register(&mut self, code: u8, value: u8) {
        match code {
            0 => self.reg.b = value,
            1 => self.reg.c = value,
            2 => self.reg.d = value,
            3 => self.reg.e = value,
            4 => self.reg.h = value,
            5 => self.reg.l = value,
            7 => self.reg.a = value,
            _ => unreachable!("code 6 is not a register"),
        }
    }

    /// Destination of a relative jump: `PC + 2 + e`, the displacement being
    /// signed.
    ///
    /// A relative jump that's **taken** loads MEMPTR with its destination; a
    /// jump not taken leaves it intact, hence the call from the taken branch only.
    fn relative_target(&mut self, displacement: u8) -> u16 {
        let target = self
            .reg
            .pc
            .wrapping_add(2)
            .wrapping_add(displacement as i8 as u16);
        self.reg.wz = target;
        target
    }

    /// MEMPTR after a memory access at `address`: the register holds the
    /// *next* address. By far the most common case.
    fn wz_after(&mut self, address: u16) {
        self.reg.wz = address.wrapping_add(1);
    }

    /// MEMPTR after `LD (nn),A`, `LD (BC),A`, `LD (DE),A` and `OUT (n),A`.
    ///
    /// These four don't follow the general rule: only the low byte advances
    /// by one, while the high byte receives `A`. The oddity really is the
    /// chip's, not a simplification on our part — it comes from the Z80
    /// presenting `A` on the high half of the address bus during that cycle.
    fn wz_after_write_a(&mut self, address: u16) {
        self.reg.wz = u16::from(self.reg.a) << 8 | u16::from((address as u8).wrapping_add(1));
    }

    /// Address targeted by an indexed access `(IX+d)`, the displacement
    /// being read as a signed byte.
    ///
    /// Any indexed access loads MEMPTR with the computed address — the
    /// broadest rule of the register, and it holds regardless of which
    /// instruction uses it.
    fn ix_d(&mut self, displacement: u8) -> u16 {
        let address = self.reg.get_ix().wrapping_add(displacement as i8 as u16);
        self.reg.wz = address;
        address
    }

    /// Same for `(IY+d)`.
    fn iy_d(&mut self, displacement: u8) -> u16 {
        let address = self.reg.get_iy().wrapping_add(displacement as i8 as u16);
        self.reg.wz = address;
        address
    }

    /// Flags common to the eight block I/O instructions (`INI`, `IND`,
    /// `OUTI`, `OUTD` and their repeated forms).
    ///
    /// S, Z and the two undocumented flags come from `B` **after**
    /// decrementing. The other three are derived from an intermediate sum
    /// the Z80 forms between the transferred byte and a second term
    /// specific to the family: `C + 1` for `INI`, `C - 1` for `IND`, and `L`
    /// (after `HL` is updated) for the two output instructions. That's the
    /// `addend` the caller supplies.
    ///
    /// These rules aren't arbitrary from the silicon's point of view — they
    /// describe an internal addition whose result is never stored anywhere
    /// — but they're reproducible, and that's exactly what the `zexall`
    /// suite checks.
    fn block_io_flags(&mut self, data: u8, addend: u8) {
        let b = self.reg.b;
        self.reg.flags.s = b & 0x80 != 0;
        self.reg.flags.z = b == 0;
        self.reg.flags.set_undocumented_from(b);
        // The one case on this machine where N doesn't systematically end
        // up at 1 after an operation that sets it: it mirrors bit 7 of the
        // transferred byte.
        self.reg.flags.n = data & 0x80 != 0;
        let k = u16::from(data) + u16::from(addend);
        self.reg.flags.h = k > 0xFF;
        self.reg.flags.c = k > 0xFF;
        self.reg.flags.p = ((k & 0x07) as u8 ^ b).count_ones() & 0x01 == 0x00;
    }

    fn ldi<B: Bus>(&mut self, bus: &mut B) {
        let bc = self.reg.get_bc();
        let de = self.reg.get_de();
        let hl = self.reg.get_hl();
        let transferred = bus.read_byte(hl);
        bus.write_byte(de, transferred);
        self.reg.set_de(de.wrapping_add(1));
        self.reg.set_hl(hl.wrapping_add(1));
        self.reg.set_bc(bc.wrapping_sub(1));
        // The two undocumented flags come from A + the transferred byte,
        // under the rule specific to block instructions: bit 3 for XF, but
        // bit 1 for YF.
        self.reg
            .flags
            .set_undocumented_from_block(self.reg.a.wrapping_add(transferred));
    }

    fn ldd<B: Bus>(&mut self, bus: &mut B) {
        let bc = self.reg.get_bc();
        let de = self.reg.get_de();
        let hl = self.reg.get_hl();
        let transferred = bus.read_byte(hl);
        bus.write_byte(de, transferred);
        self.reg.set_de(de.wrapping_sub(1));
        self.reg.set_hl(hl.wrapping_sub(1));
        self.reg.set_bc(bc.wrapping_sub(1));
        // The two undocumented flags come from A + the transferred byte,
        // under the rule specific to block instructions: bit 3 for XF, but
        // bit 1 for YF.
        self.reg
            .flags
            .set_undocumented_from_block(self.reg.a.wrapping_add(transferred));
    }

    // Returns A - (HL)
    fn cpi<B: Bus>(&mut self, bus: &mut B) {
        self.reg.wz = self.reg.wz.wrapping_add(1);
        let bc = self.reg.get_bc();
        let hl = self.reg.get_hl();
        let h = bus.read_byte(hl);
        let r = self.reg.a.wrapping_sub(h);

        self.reg.set_hl(hl.wrapping_add(1));
        self.reg.set_bc(bc.wrapping_sub(1));

        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.z = self.reg.a == h;
        self.reg.flags.h = (self.reg.a as i8 & 0x0F) < (h as i8 & 0x0F);
        self.reg.flags.p = self.reg.get_bc() != 0;
        self.reg.flags.n = true;
        // Block rule: the source is the result MINUS the half-carry, and
        // YF comes from bit 1 (see set_undocumented_from_block).
        let n = r.wrapping_sub(u8::from(self.reg.flags.h));
        self.reg.flags.set_undocumented_from_block(n);
    }

    // Returns A - (HL)
    fn cpd<B: Bus>(&mut self, bus: &mut B) {
        self.reg.wz = self.reg.wz.wrapping_sub(1);
        let bc = self.reg.get_bc();
        let hl = self.reg.get_hl();
        let h = bus.read_byte(hl);
        let r = self.reg.a.wrapping_sub(h);

        self.reg.set_hl(hl.wrapping_sub(1));
        self.reg.set_bc(bc.wrapping_sub(1));

        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.z = self.reg.a == h;
        self.reg.flags.h = (self.reg.a as i8 & 0x0F) < (h as i8 & 0x0F);
        self.reg.flags.p = self.reg.get_bc() != 0;
        self.reg.flags.n = true;
        // Block rule: the source is the result MINUS the half-carry, and
        // YF comes from bit 1 (see set_undocumented_from_block).
        let n = r.wrapping_sub(u8::from(self.reg.flags.h));
        self.reg.flags.set_undocumented_from_block(n);
    }

    // ADD A,r
    fn add(&mut self, n: u8) {
        let a = self.reg.a;
        let r = a.wrapping_add(n);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
        self.reg.flags.p = check_add_overflow(self.reg.a, n);
        self.reg.flags.h = (a & 0x0f) + (n & 0x0f) > 0x0f;
        self.reg.flags.c = u16::from(a) + u16::from(n) > 0xff;
        self.reg.flags.n = false;
        self.reg.a = r;
    }

    // ADD A,s : ADD with carry
    fn adc(&mut self, n: u8) {
        let c: u8 = match self.reg.flags.c {
            false => 0,
            true => 1,
        };
        let a = self.reg.a;
        let r = a.wrapping_add(n).wrapping_add(c);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
        self.reg.flags.p = ((a ^ r) & (n ^ r)) & 0x80 != 0;
        self.reg.flags.h = (a & 0x0f) + (n & 0x0f) + c > 0x0f;
        self.reg.flags.c = u16::from(a) + u16::from(n) + u16::from(c) > 0xff;
        self.reg.flags.n = false;
        self.reg.a = r;
    }

    // SUB s
    fn sub(&mut self, n: u8) {
        let a = self.reg.a;
        let r = a.wrapping_sub(n);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
        self.reg.flags.p = check_sub_overflow(self.reg.a, n);
        self.reg.flags.h = (a as i8 & 0x0f) < (n as i8 & 0x0f);
        self.reg.flags.c = u16::from(a) < u16::from(n);
        self.reg.flags.n = true;
        self.reg.a = r;
    }

    // SBC s
    fn sbc(&mut self, n: u8) {
        let c: u8 = match self.reg.flags.c {
            false => 0,
            true => 1,
        };
        let a = self.reg.a;
        let r = a.wrapping_sub(n).wrapping_sub(c);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
        self.reg.flags.p = ((a ^ n) & (a ^ r)) & 0x80 != 0;
        self.reg.flags.h = (a & 0x0f) < (n & 0x0f) + c;
        self.reg.flags.c = u16::from(a) < (u16::from(n) + u16::from(c));
        self.reg.flags.n = true;
        self.reg.a = r;
    }

    // Logical AND
    fn and(&mut self, n: u8) {
        let r = self.reg.a & n;
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
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
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
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
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
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
        // The Z80's best-known exception: `CP` draws its two undocumented
        // flags from the OPERAND, not from the subtraction's result —
        // unlike `SUB`, which it otherwise shares everything with. This is
        // what lets the two be told apart at runtime.
        self.reg.flags.set_undocumented_from(n);
    }

    // Increment
    fn inc(&mut self, n: u8) -> u8 {
        let r = n.wrapping_add(1);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
        self.reg.flags.p = n == 0x7F;
        self.reg.flags.h = (n & 0x0f) + 0x01 > 0x0f;
        self.reg.flags.n = false;
        r
    }

    // Decrement
    fn dec(&mut self, n: u8) -> u8 {
        let r = n.wrapping_sub(1);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
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

        if self.reg.flags.c || (self.reg.a > 0x99) {
            t += 2;
            self.reg.flags.c = true;
        }

        if self.reg.flags.n && !self.reg.flags.h {
            self.reg.flags.h = false
        } else if self.reg.flags.n && self.reg.flags.h {
            self.reg.flags.h = lsb < 6;
        } else {
            self.reg.flags.h = lsb >= 0x0A;
        }

        match t {
            1 => {
                let r = match self.reg.flags.n {
                    true => 0xFA,  // -6
                    false => 0x06, // 6
                };
                self.reg.a = self.reg.a.wrapping_add(r);
            }

            2 => {
                let r = match self.reg.flags.n {
                    true => 0xA0,  // -0x60
                    false => 0x60, // 0x60
                };
                self.reg.a = self.reg.a.wrapping_add(r);
            }

            3 => {
                let r = match self.reg.flags.n {
                    true => 0x9A,  // -0x66
                    false => 0x66, // 0x66
                };
                self.reg.a = self.reg.a.wrapping_add(r);
            }

            _ => {}
        }

        self.reg.flags.z = self.reg.a == 0x00;
        self.reg.flags.s = bit::get(self.reg.a, 7);
        self.reg.flags.set_undocumented_from(self.reg.a);
        self.reg.flags.p = self.reg.a.count_ones() & 0x01 == 0x00;
    }

    // NEG
    fn neg(&mut self) {
        let t = !self.reg.a;
        let r = t.wrapping_add(1);
        self.reg.flags.p = self.reg.a == 0x80;
        self.reg.flags.c = self.reg.a != 0;
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
        self.reg.flags.h = 0 < (self.reg.a & 0x0F);
        self.reg.flags.n = true;
        self.reg.a = r;
    }

    // 16 bits add
    fn add_16(&mut self, n1: u16, n2: u16) -> u16 {
        let r = n1.wrapping_add(n2);
        // MEMPTR takes the destination register's value BEFORE the
        // addition, plus one — for ADD HL,rr as well as ADD IX/IY,rr.
        self.reg.wz = n1.wrapping_add(1);
        self.reg.flags.c = u32::from(n1) + u32::from(n2) > 0xffff;
        self.reg.flags.h = (n1 & 0x0FFF) + (n2 & 0x0FFF) > 0x0FFF;
        self.reg.flags.n = false;
        // `ADD HL,rr` touches neither S nor Z (unlike `ADC`/`SBC HL`), but
        // it does let the two undocumented bits show through, taken from
        // the result's high byte.
        self.reg.flags.set_undocumented_from((r >> 8) as u8);
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
        self.reg.wz = h.wrapping_add(1);
        self.reg.flags.s = r & 0x8000 == 0x8000;
        // On 16-bit operations, the two undocumented flags come from the
        // HIGH byte of the result.
        self.reg.flags.set_undocumented_from((r >> 8) as u8);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.c = u32::from(h) + u32::from(n) + c as u32 > 0xffff;
        self.reg.flags.h = (h & 0x0FFF) + (n & 0x0FFF) + c > 0x0FFF;
        self.reg.flags.n = false;
        // Signed overflow: it occurs when both operands share the same
        // sign and the result ends up with a different one. Going through
        // `overflowing_add` on `n + c` was wrong twice over — `n + c`
        // overflowed for `n = 0xFFFF` with the carry set (a debug panic),
        // and the overflow of a three-term sum isn't that of the partial
        // sum.
        self.reg.flags.p = (h ^ r) & (n ^ r) & 0x8000 != 0;
    }

    // Register pair substraction with carry
    fn sbc_16(&mut self, n: u16) {
        let c: u16 = match self.reg.flags.c {
            false => 0,
            true => 1,
        };
        let h = self.reg.get_hl();
        let r = h.wrapping_sub(n).wrapping_sub(c);
        self.reg.set_hl(r);
        self.reg.wz = h.wrapping_add(1);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x8000 == 0x8000;
        // On 16-bit operations, the two undocumented flags come from the
        // HIGH byte of the result.
        self.reg.flags.set_undocumented_from((r >> 8) as u8);
        self.reg.flags.h = (h & 0x0fff) < (n & 0x0fff) + c;
        self.reg.flags.c = u32::from(h) < u32::from(n) + c as u32;
        self.reg.flags.n = true;
        // Signed overflow of a subtraction: the operands have opposite
        // signs and the result takes the subtrahend's. Same remark as for
        // `adc_16`: the version going through `n + c` truncated to `i16`
        // gave a wrong result as soon as `n + c` exceeded 0xFFFF.
        self.reg.flags.p = (h ^ n) & (h ^ r) & 0x8000 != 0;
    }

    // Rotate Accumulator left
    fn rlca(&mut self) {
        self.reg.flags.c = bit::get(self.reg.a, 7);
        let r = (self.reg.a << 1) | u8::from(self.reg.flags.c);
        self.reg.flags.c = bit::get(self.reg.a, 7);
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        self.reg.flags.set_undocumented_from(r);
        self.reg.a = r;
    }

    // Rotate left
    fn rlc(&mut self, n: u8) -> u8 {
        self.reg.flags.c = bit::get(n, 7);
        let r = (n << 1) | u8::from(self.reg.flags.c);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        r
    }

    // Rotate Accumulator right
    fn rrca(&mut self) {
        self.reg.flags.c = bit::get(self.reg.a, 0);
        let r = if self.reg.flags.c {
            0x80 | (self.reg.a >> 1)
        } else {
            self.reg.a >> 1
        };
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        self.reg.flags.set_undocumented_from(r);
        self.reg.a = r;
    }

    // Rotate right
    fn rrc(&mut self, n: u8) -> u8 {
        self.reg.flags.c = bit::get(n, 0);
        let r = if self.reg.flags.c {
            0x80 | (n >> 1)
        } else {
            n >> 1
        };
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
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
            false => self.reg.a << 1,
        };
        self.reg.flags.set_undocumented_from(r);
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
            false => n << 1,
        };
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
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
            false => self.reg.a >> 1,
        };
        self.reg.flags.set_undocumented_from(r);
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
            false => n >> 1,
        };
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        r
    }

    // Arithmetic shift left
    fn sla(&mut self, n: u8) -> u8 {
        let r = n << 1;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
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
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
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
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
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
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.set_undocumented_from(r);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0x00;
        self.reg.flags.n = false;
        self.reg.flags.c = bit::get(n, 0);
        r
    }

    // Bit test
    fn bit<B: Bus>(&mut self, bus: &mut B, operand: u8) {
        let bit = ((operand & 0x38) >> 3) as usize;
        let register = operand & 0x07;
        let r = match register {
            0 => bit::get(self.reg.b, bit),
            1 => bit::get(self.reg.c, bit),
            2 => bit::get(self.reg.d, bit),
            3 => bit::get(self.reg.e, bit),
            4 => bit::get(self.reg.h, bit),
            5 => bit::get(self.reg.l, bit),
            6 => bit::get(bus.read_byte(self.reg.get_hl()), bit),
            7 => bit::get(self.reg.a, bit),
            _ => false,
        };
        self.reg.flags.z = !r;
        self.reg.flags.h = true;
        self.reg.flags.n = false;
        // S and P/V were missing: the Z80 sets S when testing bit 7 and it's
        // 1, and mirrors Z into P/V.
        self.reg.flags.s = r && bit == 7;
        self.reg.flags.p = !r;
        // The two undocumented flags come from the TESTED VALUE — except
        // for `BIT b,(HL)`, which takes them from MEMPTR's high byte. This
        // is the only observable manifestation of that internal register,
        // and the reason it's modeled at all.
        let tested = match register {
            0 => self.reg.b,
            1 => self.reg.c,
            2 => self.reg.d,
            3 => self.reg.e,
            4 => self.reg.h,
            5 => self.reg.l,
            6 => (self.reg.wz >> 8) as u8,
            _ => self.reg.a,
        };
        self.reg.flags.set_undocumented_from(tested);
    }

    // Bit set
    fn set<B: Bus>(&mut self, bus: &mut B, operand: u8) {
        let bit = ((operand & 0x38) >> 3) as usize;
        let register = operand & 0x07;
        match register {
            0 => self.reg.b = bit::set(self.reg.b, bit),
            1 => self.reg.c = bit::set(self.reg.c, bit),
            2 => self.reg.d = bit::set(self.reg.d, bit),
            3 => self.reg.e = bit::set(self.reg.e, bit),
            4 => self.reg.h = bit::set(self.reg.h, bit),
            5 => self.reg.l = bit::set(self.reg.l, bit),
            6 => bus.write_byte(
                self.reg.get_hl(),
                bit::set(bus.read_byte(self.reg.get_hl()), bit),
            ),
            7 => self.reg.a = bit::set(self.reg.a, bit),
            _ => {}
        };
    }

    // Bit reset
    fn reset<B: Bus>(&mut self, bus: &mut B, operand: u8) {
        let bit = ((operand & 0x38) >> 3) as usize;
        let register = operand & 0x07;
        match register {
            0 => self.reg.b = bit::reset(self.reg.b, bit),
            1 => self.reg.c = bit::reset(self.reg.c, bit),
            2 => self.reg.d = bit::reset(self.reg.d, bit),
            3 => self.reg.e = bit::reset(self.reg.e, bit),
            4 => self.reg.h = bit::reset(self.reg.h, bit),
            5 => self.reg.l = bit::reset(self.reg.l, bit),
            6 => bus.write_byte(
                self.reg.get_hl(),
                bit::reset(bus.read_byte(self.reg.get_hl()), bit),
            ),
            7 => self.reg.a = bit::reset(self.reg.a, bit),
            _ => {}
        };
    }

    // call stack push
    fn call_stack_push<B: Bus>(&mut self, bus: &mut B) {
        self.reg.sp = self.reg.sp.wrapping_sub(2);
        bus.write_word(self.reg.sp, self.reg.pc.wrapping_add(3));
    }

    // call stack pop
    fn call_stack_pop<B: Bus>(&mut self, bus: &mut B) {
        self.reg.pc = bus.read_word(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(2);
        // Every return (RET, RET cc taken, RETI, RETN) goes through here,
        // and every one of them loads MEMPTR with the return address.
        self.reg.wz = self.reg.pc;
    }

    // interrupt stack push
    fn interrupt_stack_push<B: Bus>(&mut self, bus: &mut B) {
        self.reg.sp = self.reg.sp.wrapping_sub(2);
        bus.write_word(self.reg.sp, self.reg.pc);
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

#[doc(hidden)]
// Converts a signed byte to its absolute value
pub fn signed_to_abs(n: u8) -> u8 {
    !n + 1
}

/// Instruction encountered that execution doesn't know how to handle.
///
/// The processor doesn't execute it and moves on to the next one; it's up
/// to the host to decide what to do with it — display it, log it, or stop
/// on it. The `dasm` module's disassembler knows how to name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unimplemented {
    /// The instruction's address.
    pub address: u16,
    /// Its bytes, up to the longest Z80 instruction.
    pub bytes: [u8; 4],
    /// Its length in bytes.
    pub len: u8,
}
