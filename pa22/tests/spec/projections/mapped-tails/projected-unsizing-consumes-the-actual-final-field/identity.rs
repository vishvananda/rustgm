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
    tag: u8,
    tail: T::Out,
}
fn read(s: &S<[u8]>) -> u8 {
    check(s.tag == 9);
    s.tail[1]
}
fn main() {
    let s = S::<[u8; 2]> {
        tag: 9,
        tail: [3, 5],
    };
    check(read(&s) == 5);
}
