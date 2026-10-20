trait A {
    type Out: ?Sized;
}
impl<'a> A for &'a u8 {
    type Out = [u8];
}
struct S<T: A> {
    tail: T::Out,
}
trait B {}
impl<T> B for T {}
impl B for S<&'static u8> {}
fn main() {}
