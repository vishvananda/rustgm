trait A {
    type Out: ?Sized;
}
impl A for u8 {
    type Out = [u8];
}
struct S<T: A> {
    tail: T::Out,
}
struct W<T: ?Sized> {
    tail: T,
}
trait B {}
impl<T> B for T {}
impl B for W<S<u8>> {}
fn main() {}
