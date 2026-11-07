#[macro_use]
extern crate provider;
#[macro_use]
extern crate other;
fn check(ok: bool) { if !ok { let _ = fail(0); } } fn fail(x: u8) -> u8 { 1 / x }
fn main() { check(call_f!() == 9); check(provider::call_f!() == 40); }
