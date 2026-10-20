fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
fn data<T: ?Sized>(p: *const T) -> *const u8 {
    p as *const u8
}
fn main() {
    let a = [7u8, 9];
    let p: *const [u8] = &a;
    let q: *const u8 = &a[0];
    check(data(p) == q);
    check(p as *const u8 == q);
}
