trait A {
    type Out: ?Sized;
}
impl A for u8 {
    type Out = str;
}
struct S {
    tail: <u8 as A>::Out,
}
fn main() {}
