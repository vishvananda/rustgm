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
fn convert<T, const N: usize>(s: &S<[T; N]>) -> &S<[T]> {
    s
}
fn main() {
    let s = S {
        tag: 7,
        tail: [5u8, 9],
    };
    let p = convert(&s);
    check(p.tail[1] == 9);
}
