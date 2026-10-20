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
    type Out = [u8; 3];
}
impl A for u32 {
    type Out = [u8];
}
struct S<T: A> {
    tail: T::Out,
}
fn read(a: &S<u8>, b: &S<u16>, yes: bool) -> u8 {
    let s: &S<u32> = if yes { a } else { b };
    s.tail[1]
}
fn main() {
    let a = S::<u8> { tail: [3, 5] };
    let b = S::<u16> { tail: [7, 11, 13] };
    check(read(&a, &b, true) == 5);
    check(read(&a, &b, false) == 11);
}
