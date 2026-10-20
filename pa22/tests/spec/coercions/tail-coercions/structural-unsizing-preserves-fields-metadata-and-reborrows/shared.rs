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
fn read(s: &S<[u8]>) -> u8 {
    check(s.tag == 7);
    match &s.tail {
        [a, b, c] => *a + *b + *c,
        _ => 0,
    }
}
fn main() {
    let s = S {
        tag: 7,
        tail: [1, 3, 9],
    };
    check(read(&s) == 13);
}
