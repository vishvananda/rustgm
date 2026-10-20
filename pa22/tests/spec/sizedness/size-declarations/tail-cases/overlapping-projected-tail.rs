trait Value {
    type Out: ?Sized;
}
impl Value for u8 {
    type Out = u8;
}
struct S<T: Value> {
    tail: T::Out,
}
trait A {}
impl<T> A for T {}
impl A for S<u8> {}
fn main() {}
