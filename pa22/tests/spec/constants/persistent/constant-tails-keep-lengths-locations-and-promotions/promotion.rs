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
fn make() -> &'static S<[u8]> {
    &S {
        tag: 7,
        tail: [3, 5],
    }
}
fn main() {
    check(make().tag == 7);
    check(make().tail[1] == 5);
}
