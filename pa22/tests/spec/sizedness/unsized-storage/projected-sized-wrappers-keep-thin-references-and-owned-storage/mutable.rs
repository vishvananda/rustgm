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
fn write(s: &mut S<u8>) {
    s.tail = 53;
}
fn main() {
    let mut s = S::<u8> { tag: 7, tail: 31 };
    write(&mut s);
    check(s.tail == 53);
    check(s.tag == 7);
}
