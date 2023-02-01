pub fn parse_usize(slice: &[u8]) -> usize {
    assert!(slice.len() < 6);
    let mut n: usize = 0;
    for i in slice {
        n *= 10;
        n += (i - b'0') as usize;
    }
    n
}
