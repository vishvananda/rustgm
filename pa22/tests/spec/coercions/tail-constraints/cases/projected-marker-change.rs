trait A {
    type Out: ?Sized;
}
impl A for u8 {
    type Out = [u8; 2];
}
impl A for u16 {
    type Out = [u8];
}
struct S<T: A> {
    tail: T::Out,
}
fn f(s: &S<u8>) -> &S<u16> {
    s
}
fn main() {}
