fn check(ok: bool) { if !ok { let _ = fail(0); } } fn fail(x: u8) -> u8 { 1 / x }
fn main() {
    // An exported row without an allowance list keeps cfg(ub_checks) gated.
    check(internal_provider::unchecked!() == 7);
}
