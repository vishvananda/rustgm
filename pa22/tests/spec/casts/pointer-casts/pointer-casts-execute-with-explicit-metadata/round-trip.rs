fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn main() {
    let a = 7u8;
    let p: *const u8 = &a;
    let n = p as usize;
    let q = n as *const u8;
    check(p == q);
}
