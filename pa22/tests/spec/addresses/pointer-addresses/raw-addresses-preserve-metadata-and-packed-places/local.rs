fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn main() {
    let mut x = 7u8;
    let p = &raw mut x;
    x = 9;
    let q = &raw const x;
    check(q == p);
    check(x == 9);
}
