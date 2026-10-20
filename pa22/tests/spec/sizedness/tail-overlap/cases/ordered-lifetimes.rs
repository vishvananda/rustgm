trait A<'b> {
    type Out: ?Sized;
}
impl<'a: 'b, 'b> A<'b> for &'a u8 {
    type Out = [u8];
}
struct S<'b, T: A<'b>> {
    tail: T::Out,
}
trait B {}
impl<T> B for T {}
impl<'a: 'b, 'b> B for S<'b, &'a u8> {}
fn main() {}
