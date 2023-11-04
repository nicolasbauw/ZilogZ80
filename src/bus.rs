use std::{fs::File, io::{prelude::*, self}};

/// The Bus struct is hosting the Z80 memory map.
pub struct Bus {
    address_space: Vec<u8>,
    rom_space: Option<ROMSpace>,
}

/// Start and end addresses of read-only (ROM) area.
struct ROMSpace {
    pub start: u16,
    pub end: u16,
}

impl Bus {
    /// Creates a new bus instance. 'Size' will be its top address.
    pub fn new(size: u16) -> Bus {
        Bus {
            address_space: vec![0; (size as usize) + 1],
            rom_space: None,
        }
    }

    /// Sets a ROM space. Write operations will be ineffective in this address range.
    /// ```rust
    /// use zilog_z80::cpu::CPU;
    /// let mut c = CPU::new(0xFFFF);
    /// c.bus.set_romspace(0xF000, 0xFFFF);
    /// ```
    pub fn set_romspace(&mut self, start: u16, end: u16) {
        self.rom_space = Some(ROMSpace { start, end });
    }

    /// Reads a slice of bytes from memory
    pub fn read_mem_slice(&self, start: usize, end: usize) -> Vec<u8> {
        if end > self.address_space.len() {
            panic!("Read operation after the end of address space !")
        }
        self.address_space[start..end].to_vec()
    }

    /// Clears a slice of bytes in memory
    pub fn clear_mem_slice(&mut self, start: usize, end: usize) {
        if end > self.address_space.len() {
            panic!("Write operation after the end of address space !")
        }
        for m in start..=end {
            self.address_space[m] = 0;
        }
    }

    /// Reads a byte from memory
    pub fn read_byte(&self, address: u16) -> u8 {
        if address as usize >= self.address_space.len() {
            return 0;
        }
        self.address_space[usize::from(address)]
    }

    /// Writes a byte to memory
    pub fn write_byte(&mut self, address: u16, data: u8) {
        if address as usize >= self.address_space.len() {
            return;
        }
        // if rom space is declared, and write operation is requested in rom area : we exit
        if self.rom_space.is_some()
            && address >= self.rom_space.as_ref().unwrap().start
            && address <= self.rom_space.as_ref().unwrap().end
        {
            return;
        };
        self.address_space[usize::from(address)] = data;
    }

    /// Reads a word stored in memory in little endian byte order, returns this word in BE byte order
    pub fn read_word(&self, address: u16) -> u16 {
        if address as usize >= self.address_space.len() {
            return 0;
        }
        u16::from(self.address_space[usize::from(address)])
            | (u16::from(self.address_space[usize::from(address + 1)]) << 8)
    }

    /// Reads a word stored in memory in little endian byte order, returns this word in LE byte order
    pub fn read_le_word(&self, address: u16) -> u16 {
        if address as usize >= self.address_space.len() {
            return 0;
        }
        u16::from(self.address_space[usize::from(address)]) << 8
            | (u16::from(self.address_space[usize::from(address + 1)]))
    }

    /// Reads a dword stored in memory in little endian byte order, returns this dword in LE byte order
    pub fn read_le_dword(&self, address: u16) -> u32 {
        if address as usize >= self.address_space.len() {
            return 0;
        }
        u32::from(self.address_space[usize::from(address)]) << 24
            | u32::from(self.address_space[usize::from(address + 1)]) << 16
            | u32::from(self.address_space[usize::from(address + 2)]) << 8
            | u32::from(self.address_space[usize::from(address + 3)])
    }

    /// Writes a word to memory in little endian byte order
    pub fn write_word(&mut self, address: u16, data: u16) {
        if address as usize >= self.address_space.len() {
            return;
        }
        // if rom space is declared, and write operation is requested in rom area : we exit
        if self.rom_space.is_some()
            && address >= self.rom_space.as_ref().unwrap().start
            && address <= self.rom_space.as_ref().unwrap().end
        {
            return;
        };
        self.address_space[usize::from(address)] = (data & 0xFF) as u8;
        self.address_space[usize::from(address + 1)] = (data >> 8) as u8;
    }

    /// Loads binary data from disk into memory at $0000 + offset. Returns size of loaded file.
    pub fn load_bin(&mut self, file: &str, org: u16) -> io::Result<usize> {
        if org as usize >= self.address_space.len() {
            panic!("Write operation after the end of address space !")
        }
        let mut f = File::open(file)?;
        let mut buf = Vec::new();
        let s = f.read_to_end(&mut buf)?;
        self.address_space[org as usize..(buf.len() + org as usize)].clone_from_slice(&buf[..]);
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn r_le_dword() {
        let mut b = Bus::new(0xFFFF);
        b.write_byte(0x0000, 0xCC);
        b.write_byte(0x0001, 0xDD);
        b.write_byte(0x0002, 0xEE);
        b.write_byte(0x0003, 0xFF);
        assert_eq!(b.read_le_dword(0x00), 0xCCDDEEFF);
    }

    #[test]
    fn read_invalid() {
        let mut b = Bus::new(0x7FFF);
        b.write_byte(0x8000, 0xFF);
        assert_eq!(b.read_byte(0x8000), 0);
    }

    #[test]
    fn write_romspace() {
        let mut b = Bus::new(0x7FFF);
        b.write_byte(0x0000, 0xFF);
        b.set_romspace(0x0000, 0x000F);
        b.write_byte(0x0000, 0x00);
        assert_eq!(b.read_byte(0x0000), 0xFF);
    }

    #[test]
    fn clear_slice() {
        let mut b = Bus::new(0x000F);
        for m in 0..=15 {
            b.write_byte(m, 0xFF);
        }
        assert_eq!(b.read_byte(0x000F), 0xFF);
        b.clear_mem_slice(0x0000, 0x000F);
        assert_eq!(b.read_byte(0x000F), 0x00);
    }
}
