trait A {
    type Out: ?Sized;
}
struct S<T: A> {
    tail: T::Out,
}
trait B {}
impl<T> B for T {}
impl<T: A> B for S<T> {}
fn main() {}
