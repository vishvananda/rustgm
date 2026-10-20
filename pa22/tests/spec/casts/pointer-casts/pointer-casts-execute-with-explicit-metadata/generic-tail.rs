fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T: ?Sized> {
    head: u8,
    tail: T,
}
fn f<T: ?Sized>(p: *const T) -> *const S<T> {
    p as *const S<T>
}
fn main() {
    let a = [(); 3];
    let p: *const [()] = &a;
    let q = f(p);
    check(q as *const [()] == p);
}
