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
fn main() {
    let a = S {
        head: 3,
        tail: [5u8, 7],
    };
    let p = &raw const a;
    let q: *const S<[u8]> = p;
    let s: &S<[u8]> = &a;
    let r = &raw const *s;
    check(q == r);
    let t = &raw const s.tail;
    let u: *const [u8] = &s.tail;
    check(t == u);
}
