fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T: ?Sized> {
    head: u16,
    tail: T,
}
fn pass(p: *const S<[u8]>) -> *const S<[u8]> {
    p
}
fn main() {
    let a = S { head: 7, tail: [] };
    let p: *const S<[u8; 0]> = &a;
    let q: *const S<[u8]> = p;
    let s: &S<[u8]> = &a;
    let r: *const S<[u8]> = s;
    check(pass(q) == r);
}
