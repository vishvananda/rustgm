trait A {
    type Out: ?Sized;
}
impl A for [u8; 2] {
    type Out = [u8; 2];
}
impl A for [u8] {
    type Out = [u8];
}
struct S<T: A + ?Sized> {
    prefix: *const T,
    tail: T::Out,
}
fn f(s: &S<[u8; 2]>) -> &S<[u8]> {
    s
}
fn main() {}
