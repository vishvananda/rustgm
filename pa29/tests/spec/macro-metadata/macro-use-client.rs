#[macro_use]
extern crate provider;
fn f() -> u8 { 0 }
fn check(ok: bool) { if !ok { let _ = fail(0); } } fn fail(x: u8) -> u8 { 1 / x }
fn main() {
    check(call_f!() == 40);
    check(f() == 0);
    check(select!(2) == 1);
    check(nested_export!() == 2);
}
