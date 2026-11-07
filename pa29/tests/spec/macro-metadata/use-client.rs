use provider::call_f;
fn f() -> u8 { 0 }
fn check(ok: bool) { if !ok { let _ = fail(0); } } fn fail(x: u8) -> u8 { 1 / x }
fn main() {
    check(call_f!() == 40);
    check(f() == 0);
    check(provider::select!(2 | 3) == 1);
    check(provider::nested_export!(1, 2) == 5);
    check(provider::private_use() == 5);
}
