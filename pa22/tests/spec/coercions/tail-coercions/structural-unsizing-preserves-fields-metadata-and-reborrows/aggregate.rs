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
struct R<'a> {
    value: &'a S<[u8]>,
}
fn main() {
    let a = S {
        tag: 7,
        tail: [3u8, 5],
    };
    let r = R { value: &a };
    check(r.value.tail[1] == 5);
}
