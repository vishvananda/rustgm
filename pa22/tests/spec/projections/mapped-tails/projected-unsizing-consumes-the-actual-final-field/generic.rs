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
fn convert<T, const N: usize>(s: &S<[T; N]>) -> &S<[T]> {
    s
}
fn main() {
    let s = S::<[u8; 2]> { tail: [3, 5] };
    check(convert(&s).tail[1] == 5);
}
