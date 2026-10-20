trait A {
    type Out: ?Sized;
}
impl A for u8 {
    type Out = [u8];
}
struct S<T: A> {
    tail: T::Out,
}
trait B {}
impl<T: Copy> B for T {}
impl B for S<u8> {}
fn main() {}
