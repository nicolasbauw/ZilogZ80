pub fn get_bit(n: u8, b: usize) -> bool {
    (n & (1 << b)) != 0
}