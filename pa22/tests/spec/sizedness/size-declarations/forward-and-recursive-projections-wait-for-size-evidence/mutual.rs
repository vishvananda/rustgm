trait A {
    type Out: ?Sized;
}
impl<T> A for T {
    type Out = u8;
}
struct S {
    x: <U as A>::Out,
}
struct U {
    x: <S as A>::Out,
}
fn main() {}
