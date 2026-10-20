trait A {
    type Out: ?Sized;
}
impl<T> A for T {
    type Out = [u8];
}
struct S<T: A> {
    tail: T::Out,
}
trait B {}
impl<T> B for T {}
impl<T> B for S<T> {}
fn main() {}
