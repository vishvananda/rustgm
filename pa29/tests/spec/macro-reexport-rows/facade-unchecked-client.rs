fn check(ok: bool) { if !ok { let _ = fail(0); } } fn fail(x: u8) -> u8 { 1 / x }
fn main() {
    // `plain` is `unchecked`, whose row has no allowance list.
    check(facade::again::plain!() == 7);
}
