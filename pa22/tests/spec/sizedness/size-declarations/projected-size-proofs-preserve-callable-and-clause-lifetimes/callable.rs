trait Value<'a> {
    type Out: ?Sized;
}
impl<'a> Value<'a> for u8 {
    type Out = &'a u8;
}
impl<'a> Value<'a> for u16 {
    type Out = str;
}
struct S<'a, T: Value<'a>> {
    tail: T::Out,
}
fn f(_: for<'a> fn(&'a [S<'a, u8>; 2])) {}
fn main() {}
