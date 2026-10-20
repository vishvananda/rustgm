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
const P: *const S<[u8]> = &S {
    head: 5,
    tail: [3, 7],
};
fn pass(p: *const S<[u8]>) -> *const S<[u8]> {
    p
}
fn main() {
    let p = P;
    check(pass(p) == p);
}
