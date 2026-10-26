macro_rules! select { ($first:pat | $second:pat) => { 7u8 }; }
fn main() { let _ = select!(0 | 1); }
