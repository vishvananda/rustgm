struct S<const N: usize>([u8; N]);
fn read<const N: usize>(s: &S<N>, i: usize) -> u8 { s.0[i] }
fn main() { let _ = read(&S([]), 0); }
