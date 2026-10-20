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
fn edit(s: &mut S<[u8]>) {
    s.tail[2] = 23;
}
fn main() {
    let mut s = S::<[u8; 1]> {
        tag: 9,
        tail: [3, 5, 7],
    };
    edit(&mut s);
    check(s.tail[2] == 23);
}
