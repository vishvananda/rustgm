fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn pass(p: *const [u8]) -> *const [u8] {
    p
}
fn main() {
    let a = [3u8, 5, 7];
    let p: *const [u8; 3] = &a;
    let q: *const [u8] = p;
    let s: &[u8] = &a;
    let r: *const [u8] = s;
    check(pass(q) == r);
}
