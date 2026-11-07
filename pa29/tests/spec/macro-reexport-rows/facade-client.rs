fn check(ok: bool) { if !ok { let _ = fail(0); } } fn fail(x: u8) -> u8 { 1 / x }
fn main() {
    // The defining row's allowance list follows its identity through re-exports.
    check(facade::checked!() == 7);
    check(facade::again::twice!() == 7);
}
