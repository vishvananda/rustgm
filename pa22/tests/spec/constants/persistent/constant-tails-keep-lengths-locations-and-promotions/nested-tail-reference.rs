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
struct W<T: ?Sized> {
    prefix: u32,
    tail: S<T>,
}
static B: W<[u8; 2]> = W {
    prefix: 17,
    tail: S {
        tag: 7,
        tail: [3, 5],
    },
};
const C: &W<[u8]> = &B;
const T: &S<[u8]> = &C.tail;
fn main() {
    check(C.prefix == 17);
    check(T.tag == 7);
    check(T.tail[1] == 5);
}
