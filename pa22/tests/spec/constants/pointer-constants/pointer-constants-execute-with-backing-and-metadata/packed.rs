fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
#[repr(packed)]
struct S {
    a: u8,
    b: u16,
}
static X: S = S { a: 7, b: 9 };
const P: *const u16 = &raw const X.b;
fn main() {
    let q = &raw const X.b;
    check(P == q);
}
