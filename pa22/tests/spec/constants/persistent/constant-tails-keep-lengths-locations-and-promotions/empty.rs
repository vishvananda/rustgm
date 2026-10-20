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
const C: &S<[u8]> = &S { tag: 7, tail: [] };
fn main() {
    check(C.tag == 7);
    match &C.tail {
        [] => (),
        _ => check(false),
    }
}
