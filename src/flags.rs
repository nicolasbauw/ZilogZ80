// Status Indicator Flags
pub struct Flags {
    pub s: bool,  // sign                 : bit 7
    pub z: bool,  // zero                 : bit 6
    pub b5: bool, // unused
    pub h: bool,  // auxiliary carry      : bit 4
    pub b3: bool, // unused
    pub p: bool,  // parity / overflow    : bit 2
    pub n: bool,  // substract            : bit 1
    pub c: bool,  // carry                : bit 0
}

impl Default for Flags {
    fn default() -> Self {
        Self::new()
    }
}

impl Flags {
    pub fn new() -> Flags {
        Flags {
            s: false,
            z: false,
            b5: false,
            h: false,
            b3: false,
            p: false,
            n: false,
            c: false,
        }
    }

    /// Recopie les deux drapeaux non documentés (bits 3 et 5, dits XF et YF)
    /// depuis une valeur — le résultat de l'opération dans la grande
    /// majorité des cas.
    ///
    /// Ces bits n'ont aucune signification propre : le Z80 y laisse
    /// simplement transparaître les bits correspondants de sa dernière
    /// opération. Ils ne sont observables qu'en passant par `PUSH AF`, mais
    /// certaines protections de copie s'en servent précisément pour cette
    /// raison — d'où l'intérêt de les émuler fidèlement.
    ///
    /// Attention aux exceptions, qui ne prennent pas le résultat comme
    /// source : `CP` les tire de l'opérande, `BIT n,(HL)` de `MEMPTR`, et
    /// les instructions de bloc d'une somme intermédiaire (voir
    /// `set_undocumented_from_block`).
    pub fn set_undocumented_from(&mut self, value: u8) {
        self.b3 = value & 0x08 != 0;
        self.b5 = value & 0x20 != 0;
    }

    /// Variante propre aux instructions de bloc (`LDI`/`LDD`, `CPI`/`CPD` et
    /// leurs formes répétitives) : là, le bit 3 de la valeur donne XF, mais
    /// c'est le **bit 1** qui donne YF, et non le bit 5.
    pub fn set_undocumented_from_block(&mut self, value: u8) {
        self.b3 = value & 0x08 != 0;
        self.b5 = value & 0x02 != 0;
    }

    /// Converts Status Indicator Flags to a byte.
    pub fn to_byte(&self) -> u8 {
        let s = if self.s { 1 << 7 } else { 0 };
        let z = if self.z { 1 << 6 } else { 0 };
        let b5 = if self.b5 { 1 << 5 } else { 0 };
        let h = if self.h { 1 << 4 } else { 0 };
        let b3 = if self.b3 { 1 << 3 } else { 0 };
        let p = if self.p { 1 << 2 } else { 0 };
        let n = if self.n { 1 << 1 } else { 0 };
        let c = if self.c { 1 } else { 0 };
        s | z | b5 | h | b3 | p | n | c
    }

    /// Retrieves Status Indicator Flags from a byte.
    pub fn set_from_byte(&mut self, bflags: u8) {
        self.s = (bflags & 0x80) != 0;
        self.z = (bflags & 0x40) != 0;
        self.b5 = (bflags & 0x20) != 0;
        self.h = (bflags & 0x10) != 0;
        self.b3 = (bflags & 0x08) != 0;
        self.p = (bflags & 0x04) != 0;
        self.n = (bflags & 0x02) != 0;
        self.c = (bflags & 0x01) != 0;
    }
}
