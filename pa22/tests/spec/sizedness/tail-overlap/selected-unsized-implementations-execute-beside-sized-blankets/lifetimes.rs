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
impl<'a> A for &'a u8 {
    type Out = [u8];
}
struct S<T: A> {
    tail: T::Out,
}
trait B {
    fn n() -> u8;
}
impl<T> B for T {
    fn n() -> u8 {
        1
    }
}
impl<'a> B for S<&'a u8> {
    fn n() -> u8 {
        4
    }
}
fn f<'a>(_: &'a u8) -> u8 {
    <S<&'a u8> as B>::n()
}
fn main() {
    check(f(&7) == 4);
    check(<u8 as B>::n() == 1);
}
