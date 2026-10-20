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
fn choose<'a>(ok: bool, a: &'a S<[u8; 2]>, b: &'a S<[u8; 3]>) -> &'a S<[u8]> {
    if ok { a } else { b }
}
fn main() {
    let a = S {
        tag: 7,
        tail: [3, 5],
    };
    let b = S {
        tag: 9,
        tail: [7, 11, 13],
    };
    check(choose(false, &a, &b).tail[2] == 13);
}
