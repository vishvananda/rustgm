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
impl<const N: usize> A for [u8; N] {
    type Out = [u16; 3];
}
impl A for [u8] {
    type Out = [u16];
}
struct S<T: A + ?Sized> {
    tag: u8,
    tail: T::Out,
}
fn read(s: &S<[u8]>) -> u16 {
    match &s.tail {
        [a, b, c] => *a + *b + *c,
        _ => 0,
    }
}
fn main() {
    let s = S::<[u8; 1]> {
        tag: 9,
        tail: [3, 5, 7],
    };
    check(read(&s) == 15);
}
