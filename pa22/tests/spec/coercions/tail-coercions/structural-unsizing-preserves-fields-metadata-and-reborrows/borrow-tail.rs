fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T: ?Sized> {
    tag: u16,
    tail: T,
}
fn read(s: &S<[u8]>) -> &[u8] {
    &s.tail
}
fn main() {
    let a = S {
        tag: 7,
        tail: [3, 5],
    };
    let p = read(&a);
    check(p[1] == 5);
}
