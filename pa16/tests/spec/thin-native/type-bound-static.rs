fn check(ok: bool) { if !ok { let _ = fail(0); } }
fn fail(x: u8) -> u8 { 1 / x }
static VALUE: u16 = 65531;
fn persistent<'a>(x: &'a u16) -> &'static u16 where &'a u16: 'static { x }
fn main() { let r = persistent(&VALUE); check(*r == 65531); }
