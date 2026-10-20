trait A<const N: usize> {
    type Out: ?Sized;
}
impl A<2> for u8 {
    type Out = [u16; 3];
}
impl A<7> for u16 {
    type Out = [u16];
}
struct S<T: A<N>, const N: usize> {
    prefix: [u8; N],
    tail: T::Out,
}
fn f(s: &S<u8, 2>) -> &S<u16, 7> {
    s
}
fn main() {}
