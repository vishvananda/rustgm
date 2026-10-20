fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn main() {
    let s = b"abc";
    let p: *const [u8] = s;
    let q: *const [u8] = s;
    check(p == q);
}
