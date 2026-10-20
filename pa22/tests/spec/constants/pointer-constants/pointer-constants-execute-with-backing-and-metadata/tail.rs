fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T: ?Sized> {
    head: u16,
    tail: T,
}
static X: S<[u8; 2]> = S {
    head: 3,
    tail: [5, 7],
};
const P: *const S<[u8]> = (&raw const X) as *const S<[u8]>;
fn main() {
    let q: *const S<[u8]> = &X;
    check(P == q);
}
