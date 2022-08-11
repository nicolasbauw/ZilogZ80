pub struct AddressBus {
    address_space: Vec<u8>,
    pub rom_space: Option<ROMSpace>,
}

/// Start and end addresses of read-only (ROM) area.
pub struct ROMSpace {
    pub start: u16,
    pub end: u16,
}

impl AddressBus {
    pub fn new() -> AddressBus {
        AddressBus {
            address_space: vec![0; 65536],
            rom_space: None,
        }
    }

    /// Reads a byte from memory
    pub fn read_byte(&self, address: u16) -> u8 {
        self.address_space[usize::from(address)]
    }

    /// Writes a byte to memory
    pub fn write_byte(&mut self, address: u16, data: u8) {
        // if rom space is declared, and write operation is requested in rom area : we exit
        if self.rom_space.is_some() && address >= self.rom_space.as_ref().unwrap().start && address <= self.rom_space.as_ref().unwrap().end { return };
        self.address_space[usize::from(address)] = data;
    }

    /// Reads a word stored in memory in little endian byte order, returns this word in BE byte order
    pub fn read_word(&self, address: u16) -> u16 {
        u16::from(self.address_space[usize::from(address)]) | (u16::from(self.address_space[usize::from(address + 1)]) << 8)
    }

    // Reads a word stored in memory in little endian byte order, returns this word in LE byte order
    pub fn read_le_word(&self, address: u16) -> u16 {
        u16::from(self.address_space[usize::from(address)]) << 8 | (u16::from(self.address_space[usize::from(address + 1)]))
    }

    /// Writes a word to memory in little endian byte order
    pub fn write_word(&mut self, address: u16, data: u16) {
        // if rom space is declared, and write operation is requested in rom area : we exit
        if self.rom_space.is_some() && address >= self.rom_space.as_ref().unwrap().start && address <= self.rom_space.as_ref().unwrap().end { return };
        self.address_space[usize::from(address)] = (data & 0xFF) as u8;
        self.address_space[usize::from(address + 1)] = (data >> 8) as u8;
    }
}