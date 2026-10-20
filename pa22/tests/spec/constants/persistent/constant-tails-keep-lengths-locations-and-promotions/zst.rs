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
const C: &S<[()]> = &S {
    tag: 7,
    tail: [(); 3],
};
fn main() {
    check(C.tag == 7);
    match &C.tail {
        [(), (), ()] => (),
        _ => check(false),
    }
}
