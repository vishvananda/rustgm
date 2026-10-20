fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn echo(p: *const [u8]) -> *const [u8] {
    &raw const *p
}
fn main() {
    let a = [3u8, 5];
    let s: &[u8] = &a;
    let p = &raw const *s;
    let q: *const [u8] = s;
    check(echo(p) == q);
}
