fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
#[repr(packed)]
struct P {
    a: u8,
    b: u16,
}
fn main() {
    let mut p = P { a: 3, b: 7 };
    let x = &raw const p.b;
    let y = &raw mut p.b;
    p.b = 9;
    check(x == y);
    check(p.b == 9);
}
