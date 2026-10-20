fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
trait A<const N: usize> {
    type Out: ?Sized;
}
impl A<2> for u8 {
    type Out = [u16; 3];
}
impl A<7> for u16 {
    type Out = [u16];
}
struct S<T: A<N>, const N: usize> {
    tag: u8,
    tail: T::Out,
}
fn read(s: &S<u16, 7>) -> u16 {
    check(s.tag == 9);
    s.tail[2]
}
fn main() {
    let s = S::<u8, 2> {
        tag: 9,
        tail: [3, 5, 7],
    };
    check(read(&s) == 7);
}
