fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn main() {
    let a = [7u16, 9];
    let p: *const [u16] = &a;
    let q = p as *const [u8];
    let thin = q as *const u8;
    let array = thin as *const [u8; 4];
    let r: *const [u8] = array;
    check(q != r);
    let two = thin as *const [u8; 2];
    let s: *const [u8] = two;
    check(q == s);
}
