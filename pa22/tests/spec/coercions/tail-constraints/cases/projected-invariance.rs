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
fn f<'a, 'b: 'a>(s: &'a mut S<&'b u8>) -> &'a mut S<&'a u16> {
    s
}
fn main() {}
