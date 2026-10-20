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
impl A for u8 {
    type Out = [u8; 2];
}
impl A for u16 {
    type Out = [u8];
}
struct S<T: A> {
    tag: u8,
    tail: T::Out,
}
fn read(s: &S<u16>) -> u8 {
    check(s.tag == 9);
    s.tail[1]
}
fn main() {
    check(
        read(&S::<u8> {
            tag: 9,
            tail: [3, 5],
        }) == 5,
    );
}
