fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn pick(ok: bool, a: &[u8; 2], b: &[u8; 3]) -> *const [u8] {
    if ok { a } else { b }
}
fn main() {
    let a = [3u8, 5];
    let b = [7u8, 9, 11];
    let p: *const [u8] = &a;
    let q: *const [u8] = &b;
    check(pick(true, &a, &b) == p);
    check(pick(false, &a, &b) == q);
}
