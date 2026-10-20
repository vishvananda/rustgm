fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct D;
impl Drop for D {
    fn drop(&mut self) {
        let _ = fail(0);
    }
}
struct S<T: ?Sized> {
    tag: u8,
    tail: T,
}
fn read(s: &S<[D]>) -> u8 {
    s.tag
}
fn main() {
    let s = S { tag: 7, tail: [D] };
    check(read(&s) == 7);
}
