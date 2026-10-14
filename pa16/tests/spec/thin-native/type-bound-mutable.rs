fn check(ok: bool) { if !ok { let _ = fail(0); } }
fn fail(x: u8) -> u8 { 1 / x }
fn shorten<'a, 'b>(x: &'a mut u16) -> &'b mut u16 where [&'a mut u16; 0]: 'b { x }
fn main() { let mut x = 11u16; { let r = shorten(&mut x); *r += 7; check(*r == 18); } check(x == 18); }
