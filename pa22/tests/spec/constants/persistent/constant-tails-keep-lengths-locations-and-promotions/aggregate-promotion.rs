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
struct R {
    p: &'static S<[u8]>,
}
fn make() -> &'static R {
    &R {
        p: &S {
            tag: 7,
            tail: [3, 5],
        },
    }
}
fn main() {
    check(make().p.tail[1] == 5);
}
