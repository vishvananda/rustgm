fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T: ?Sized> {
    tail: T,
}
fn main() {
    let a = S { tail: [7u8, 9] };
    let p: *const S<[u8; 2]> = &a;
    let q = p as *const S<[u8]>;
    let r: *const S<[u8]> = &a;
    check(q == r);
    let p = &a.tail as *const [u8];
    let q: *const [u8] = &a.tail;
    check(p == q);
}
