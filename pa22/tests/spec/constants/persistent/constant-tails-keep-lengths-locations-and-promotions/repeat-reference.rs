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
static A: [&S<[u8]>; 2] = [C; 2];
fn main() {
    check(A[0].tag == 7);
    check(A[1].tail[1] == 5);
}
