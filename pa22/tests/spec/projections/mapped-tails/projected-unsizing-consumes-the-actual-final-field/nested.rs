fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
trait A {
    type Out: ?Sized;
}
impl<T, const N: usize> A for [T; N] {
    type Out = [T; N];
}
impl<T> A for [T] {
    type Out = [T];
}
struct S<T: A + ?Sized> {
    tail: T::Out,
}
struct W<T: A + ?Sized> {
    tag: u8,
    inner: S<T>,
}
fn read(s: &W<[u8]>) -> u8 {
    check(s.tag == 9);
    s.inner.tail[1]
}
fn main() {
    let s = W::<[u8; 2]> {
        tag: 9,
        inner: S::<[u8; 2]> { tail: [3, 5] },
    };
    check(read(&s) == 5);
}
