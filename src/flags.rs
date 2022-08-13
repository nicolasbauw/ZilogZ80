// Status Indicator Flags
pub struct Flags {
    pub s: bool,                // sign                 : bit 7
    pub z: bool,                // zero                 : bit 6
    pub h: bool,                // auxiliary carry      : bit 4
    pub p: bool,                // parity / overflow    : bit 2
    pub n: bool                 // substract            : bit 1
    pub c: bool                 // carry                : bit 0
}

impl Flags {
    pub fn new() -> Flags {
        Flags {
            s: false,
            z: false,
            h: false,
            p: false,
            n: false,
            c: false
        }
    }
}