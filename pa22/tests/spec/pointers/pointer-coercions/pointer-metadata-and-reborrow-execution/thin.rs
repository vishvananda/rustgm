fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn pass(p: *const u8) -> *const u8 {
    p
}
fn main() {
    let mut x = 7u8;
    let r = &mut x;
    let p: *mut u8 = r;
    *r = 9;
    let q: *const u8 = &x;
    check(pass(p) == q);
    check(x == 9);
}
