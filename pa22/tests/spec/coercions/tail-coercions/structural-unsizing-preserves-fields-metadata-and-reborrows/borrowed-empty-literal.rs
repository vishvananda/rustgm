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
fn read(s: &S<[u8]>) -> bool {
    match &s.tail {
        [] => true,
        _ => false,
    }
}
fn main() {
    check(read(&S { tag: 7, tail: [] }));
}
