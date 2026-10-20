fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
static X: u8 = 7;
const P: *const u8 = &raw const X;
fn main() {
    let q = &raw const X;
    check(P == q);
}
