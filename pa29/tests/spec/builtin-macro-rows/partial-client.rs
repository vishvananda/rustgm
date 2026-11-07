fn check(ok: bool) { if !ok { let _ = fail(0); } } fn fail(x: u8) -> u8 { 1 / x }
fn main() {
    // The row allows cfg_ub_checks only; its atomic cfg stays gated.
    check(internal_provider::partial!() == 7);
}
