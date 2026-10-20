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
fn main() {
    let a = S {
        tag: 7,
        tail: [3u8, 5],
    };
    let b = S {
        tag: 9,
        tail: [7, 11, 13],
    };
    let mut p: &S<[u8]> = &a;
    p = &b;
    check(p.tail[2] == 13);
}
