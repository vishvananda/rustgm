fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn main() {
    let mut a = [7u8, 9];
    let r = &mut a;
    let p = r as *mut u8;
    r[0] = 11;
    let q: *const u8 = &a[0];
    check(q == p);
    check(a[0] == 11);
}
