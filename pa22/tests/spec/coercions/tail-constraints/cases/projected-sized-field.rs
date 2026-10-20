trait A {
    type Out: ?Sized;
}
impl A for [u8; 2] {
    type Out = u8;
}
impl A for [u8] {
    type Out = u8;
}
struct S<T: A + ?Sized> {
    tail: T::Out,
}
fn f(s: &S<[u8; 2]>) -> &S<[u8]> {
    s
}
fn main() {}
