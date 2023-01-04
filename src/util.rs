pub fn parse_usize(slice: &[u8]) -> usize {
    let mut n: usize = 0;
    for i in slice {
        n *= 10;
        n += (i - b'0') as usize;
    }
    n
}
