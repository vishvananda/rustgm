fn check(ok: bool) { if !ok { let _ = fail(0); } } fn fail(x: u8) -> u8 { 1 / x }
use facade::api::*;
fn main() { check(make().value == 7); check(*&DATA == 7); }
