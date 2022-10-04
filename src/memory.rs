use std::{fs::File, io::prelude::*,};

/// The AddressBus struct is hosting the Z80 memory map.
pub struct AddressBus {
    address_space: Vec<u8>,
    rom_space: Option<ROMSpace>,
    pub rw: (crossbeam_channel::Sender<(u16, Vec<u8>)>, crossbeam_channel::Receiver<(u16, Vec<u8>)>)
}

/// Start and end addresses of read-only (ROM) area.
struct ROMSpace {
    pub start: u16,
    pub end: u16,
}

impl AddressBus {
    pub fn new() -> AddressBus {
        AddressBus {
            address_space: vec![0; 65536],
            rom_space: None,
            rw: crossbeam_channel::bounded(1),
        }
    }

    /// Sets a ROM space. Write operations will be ineffective in this address range.
    /// ```rust
    /// use zilog_z80::cpu::CPU;
    /// let mut c = CPU::new();
    /// c.bus.set_romspace(0xF000, 0xFFFF);
    /// ```
    pub fn set_romspace(&mut self, start: u16, end: u16) {
        self.rom_space = Some(ROMSpace{start, end});
    }

    /// Send a vec of bytes of the address space to rw channel
    pub fn channel_send(&self, start: usize, end: usize) {
        let mut d: Vec<u8> = Vec::new();
        for i in 0..end-start {
            d.push(self.address_space[i]);
        }
        self.rw.0.send((start as u16, d)).unwrap();
    }

    /// Reads a slice of bytes
    pub fn read_mem_slice(&self, start: usize, end: usize) -> &[u8] {
        &self.address_space[start..end]
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

    /// Reads a word stored in memory in little endian byte order, returns this word in LE byte order
    pub fn read_le_word(&self, address: u16) -> u16 {
        u16::from(self.address_space[usize::from(address)]) << 8 | (u16::from(self.address_space[usize::from(address + 1)]))
    }

    /// Reads a dword stored in memory in little endian byte order, returns this dword in LE byte order
    pub fn read_le_dword(&self, address: u16) -> u32 {
        u32::from(self.address_space[usize::from(address)]) << 24 |
        u32::from(self.address_space[usize::from(address + 1)]) << 16 |
        u32::from(self.address_space[usize::from(address + 2)]) << 8 |
        u32::from(self.address_space[usize::from(address + 3)])
    }

    /// Writes a word to memory in little endian byte order
    pub fn write_word(&mut self, address: u16, data: u16) {
        // if rom space is declared, and write operation is requested in rom area : we exit
        if self.rom_space.is_some() && address >= self.rom_space.as_ref().unwrap().start && address <= self.rom_space.as_ref().unwrap().end { return };
        self.address_space[usize::from(address)] = (data & 0xFF) as u8;
        self.address_space[usize::from(address + 1)] = (data >> 8) as u8;
    }

    /// Loads binary data from disk into memory at $0000 + offset
    pub fn load_bin(&mut self, file: &str, org: u16) -> Result<(), std::io::Error> {
        let mut f = File::open(file)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        self.address_space[org as usize..(buf.len() + org as usize)].clone_from_slice(&buf[..]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn r_le_dword() {
        let mut b = AddressBus::new();
        b.write_byte(0x0000, 0xCC);
        b.write_byte(0x0001, 0xDD);
        b.write_byte(0x0002, 0xEE);
        b.write_byte(0x0003, 0xFF);
        assert_eq!(b.read_le_dword(0x00), 0xCCDDEEFF);
    }
}