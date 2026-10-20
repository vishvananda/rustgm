fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
trait Value {
    type Out: ?Sized;
}
impl Value for u8 {
    type Out = u16;
}
struct S<T: Value> {
    tag: u8,
    tail: T::Out,
}
fn read(s: &[S<u8>; 2]) -> u16 {
    s[0].tail + s[1].tail
}
fn main() {
    let values = [S::<u8> { tag: 1, tail: 11 }, S::<u8> { tag: 2, tail: 19 }];
    check(read(&values) == 30);
}
