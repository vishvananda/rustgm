trait A {
    type Out;
}
impl<T> A for T {
    type Out = *const T;
}
struct S {
    x: <S as A>::Out,
}
fn main() {}
