fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
trait A {
    type Out: ?Sized;
}
impl A for u8 {
    type Out = [u8; 2];
}
impl A for u16 {
    type Out = [u8];
}
struct S<T: A> {
    head: u16,
    tail: T::Out,
}
fn pass(p: *const S<u16>) -> *const S<u16> {
    p
}
fn main() {
    let s = S::<u8> {
        head: 5,
        tail: [3, 7],
    };
    let p: *const S<u8> = &s;
    let q: *const S<u16> = p;
    let r: &S<u16> = &s;
    let t: *const S<u16> = r;
    check(pass(q) == t);
}
