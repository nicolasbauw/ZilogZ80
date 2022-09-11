use crate::flags::Flags;

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub ixh: u8,
    pub ixl: u8,
    pub iyh: u8,
    pub iyl: u8,
    pub flags: Flags
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            ixh: 0,
            ixl: 0,
            iyh: 0,
            iyl: 0,
            flags: Flags::new()
        }
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }

    pub fn get_ix(&self) -> u16 {
        (self.ixh as u16) << 8 | self.ixl as u16
    }

    pub fn set_ix(&mut self, value: u16) {
        self.ixh = ((value & 0xFF00) >> 8) as u8;
        self.ixl = (value & 0xFF) as u8;
    }

    pub fn get_iy(&self) -> u16 {
        (self.iyh as u16) << 8 | self.iyl as u16
    }

    pub fn set_iy(&mut self, value: u16) {
        self.iyh = ((value & 0xFF00) >> 8) as u8;
        self.iyl = (value & 0xFF) as u8;
    }

    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | self.flags.to_byte() as u16
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.flags.from_byte((value & 0xFF) as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_set_ix() {
        let mut registers = Registers::new();
        registers.set_ix(0xF5A2);
        assert_eq!(registers.ixh, 0xF5);
        assert_eq!(registers.ixl, 0xA2);
        assert_eq!(registers.get_ix(), 0xF5A2);
    }

    #[test]
    fn get_set_iy() {
        let mut registers = Registers::new();
        registers.set_iy(0xF5A2);
        assert_eq!(registers.iyh, 0xF5);
        assert_eq!(registers.iyl, 0xA2);
        assert_eq!(registers.get_iy(), 0xF5A2);
    }
}