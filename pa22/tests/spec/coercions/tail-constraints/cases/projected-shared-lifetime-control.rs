trait A {
    type Out: ?Sized;
}
impl<'a> A for &'a u8 {
    type Out = [&'a u8; 2];
}
impl<'a> A for &'a u16 {
    type Out = [&'a u8];
}
struct S<T: A> {
    tail: T::Out,
}
fn f<'a>(s: &'a S<&'a u8>) -> &'a S<&'a u16> {
    s
}
fn main() {}
