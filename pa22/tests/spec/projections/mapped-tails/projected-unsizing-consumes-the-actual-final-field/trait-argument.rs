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
impl<P, const N: usize> A<P> for [u8; N] {
    type Out = [P; N];
}
impl<P> A<P> for [u8] {
    type Out = [P];
}
struct S<P, T: A<P> + ?Sized> {
    tail: T::Out,
}
fn read(s: &S<u16, [u8]>) -> u16 {
    s.tail[1]
}
fn main() {
    let s = S::<u16, [u8; 2]> { tail: [3, 5] };
    check(read(&s) == 5);
}
