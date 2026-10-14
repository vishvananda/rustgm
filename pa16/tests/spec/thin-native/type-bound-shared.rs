fn check(ok: bool) { if !ok { let _ = fail(0); } }
fn fail(x: u8) -> u8 { 1 / x }
fn shorten<'a, 'b>(x: &'a u16) -> &'b u16 where (&'a u16,): 'b { x }
fn main() { let mut x = 519u16; { let r = shorten(&x); check(*r == 519); } x = 821; check(x == 821); }
