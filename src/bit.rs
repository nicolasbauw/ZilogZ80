pub fn get_bit(n: u8, b: usize) -> bool {
    (n & (1 << b)) != 0
}

pub fn set_bit(n: u8, b: usize) -> u8 {
    n | (1 << b)
}