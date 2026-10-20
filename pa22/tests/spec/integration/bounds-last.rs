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
fn read(s: &S<[u8]>, i: usize) -> u8 {
    s.tail[i]
}
fn main() {
    check(
        read(
            &S {
                tag: 1,
                tail: [3, 7],
            },
            1,
        ) == 7,
    );
}
