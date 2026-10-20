trait A {
    type Out: ?Sized;
}
impl<T: 'static> A for T {
    type Out = [u8];
}
struct S<T: A> {
    tail: T::Out,
}
trait B {}
impl<T> B for T {}
impl<'a> B for S<&'a u8> {}
fn main() {}
