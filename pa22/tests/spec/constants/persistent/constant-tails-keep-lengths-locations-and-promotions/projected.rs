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
    type Out = [u16; 3];
}
impl A for u16 {
    type Out = [u16];
}
struct S<T: A> {
    tag: u8,
    tail: T::Out,
}
const C: &S<u16> = &S::<u8> {
    tag: 7,
    tail: [3, 5, 7],
};
fn main() {
    check(C.tag == 7);
    check(C.tail[2] == 7);
}
