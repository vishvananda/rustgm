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
impl B for S<u8> {
    fn n() -> u8 {
        2
    }
}
fn main() {
    check(<S<u8> as B>::n() == 2);
    check(<u8 as B>::n() == 1);
}
