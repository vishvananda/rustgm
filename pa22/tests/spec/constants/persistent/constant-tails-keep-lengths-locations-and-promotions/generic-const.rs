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
struct G<const N: usize>;
impl<const N: usize> G<N> {
    const C: &'static S<[u8]> = &S {
        tag: 7,
        tail: [5; N],
    };
}
fn main() {
    check(G::<2>::C.tag == 7);
    match &G::<2>::C.tail {
        [a, b] => check(*a == 5 && *b == 5),
        _ => check(false),
    }
}
