struct S([u16; 0]);
fn write(mut s: S, i: usize) { s.0[i] = 7; }
fn main() { write(S([]), 0); }
