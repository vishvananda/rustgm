trait A {
    type Out: ?Sized;
}
impl<T> A for T {
    type Out = u8;
}
struct S {
    x: <S as A>::Out,
}
fn main() {}
