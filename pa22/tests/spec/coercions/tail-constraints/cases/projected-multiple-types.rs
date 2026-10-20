trait A<P> {
    type Out: ?Sized;
}
impl A<u8> for u8 {
    type Out = [u16; 3];
}
impl A<u16> for u16 {
    type Out = [u16];
}
struct S<P, T: A<P>> {
    tail: T::Out,
}
fn f(s: &S<u8, u8>) -> &S<u16, u16> {
    s
}
fn main() {}
