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
struct W<T: ?Sized>(u8, T);
fn read(s: &W<[u8]>) -> u8 {
    s.1[1]
}
fn main() {
    check(read(&W(7, [3, 5])) == 5);
}
