fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T: ?Sized> {
    tag: u16,
    tail: T,
}
struct W<T: ?Sized> {
    prefix: u32,
    inner: S<T>,
}
fn read(s: &W<[u8]>) -> u8 {
    check(s.prefix == 31);
    check(s.inner.tag == 7);
    s.inner.tail[2]
}
fn main() {
    let s = W {
        prefix: 31,
        inner: S {
            tag: 7,
            tail: [1, 3, 9],
        },
    };
    check(read(&s) == 9);
}
