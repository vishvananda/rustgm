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
    inner: S<T>,
}
fn read(s: &W<[u8]>) -> u8 {
    s.inner.tail[1]
}
fn main() {
    check(
        read(&W {
            inner: S {
                tag: 7,
                tail: [3, 5],
            },
        }) == 5,
    );
}
