fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn g() {}
fn main() {
    let f: fn() = g;
    let p = g as *const u8;
    let q = f as *const u8;
    check(p == q);
    check(p as usize == g as usize);
}
