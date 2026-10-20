trait A {
    type Out;
}
impl<T> A for T {
    type Out = u8;
}
type X = <S as A>::Out;
struct S {
    x: u16,
}
fn f(_: X) {}
fn main() {}
