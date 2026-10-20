trait Value {
    type Out: ?Sized;
}
impl Value for u8 {
    type Out = u16;
}
impl Value for u16 {
    type Out = str;
}
struct S<T: Value> {
    tag: u8,
    tail: T::Out,
}
struct P {
    values: [S<u16>; 0],
}
fn main() {}
