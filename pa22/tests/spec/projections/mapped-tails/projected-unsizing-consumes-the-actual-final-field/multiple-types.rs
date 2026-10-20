fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
trait A<P> {
    type Out: ?Sized;
}
impl A<u8> for u8 {
    type Out = [u16; 3];
}
impl A<u16> for u16 {
    type Out = [u16];
}
struct S<P, T: A<P>> {
    tag: u8,
    tail: T::Out,
}
fn read(s: &S<u16, u16>) -> u16 {
    check(s.tag == 9);
    s.tail[2]
}
fn main() {
    let s = S::<u8, u8> {
        tag: 9,
        tail: [3, 5, 7],
    };
    check(read(&s) == 7);
}
