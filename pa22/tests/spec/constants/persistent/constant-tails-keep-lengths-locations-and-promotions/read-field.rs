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
const C: &S<[u8]> = &S {
    tag: 7,
    tail: [3, 5],
};
const N: u8 = C.tail[1];
fn main() {
    check(N == 5);
}
