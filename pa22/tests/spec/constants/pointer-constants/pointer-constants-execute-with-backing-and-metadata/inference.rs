fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn f(x: &[u8; 2]) -> *const [u8] {
    x as _
}
fn g(x: &[u8; 2]) -> *const u8 {
    x as _
}
fn main() {
    let a = [7u8, 9];
    let p: *const [u8] = &a;
    let q: *const u8 = &a[0];
    check(f(&a) == p);
    check(g(&a) == q);
}
