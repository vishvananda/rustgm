fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T: ?Sized> {
    tag: u8,
    tail: T,
}
trait A {
    const C: &'static S<[u8]>;
}
impl A for u8 {
    const C: &'static S<[u8]> = &S {
        tag: 7,
        tail: [3, 5],
    };
}
fn main() {
    check(u8::C.tag == 7);
    check(u8::C.tail[1] == 5);
}
