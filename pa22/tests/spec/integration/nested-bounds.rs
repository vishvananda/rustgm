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
    tail: S<T>,
}
fn read(s: &W<[u8]>, i: usize) -> u8 {
    s.tail.tail[i]
}
fn main() {
    let _ = read(
        &W {
            tail: S {
                tag: 1,
                tail: [3, 7],
            },
        },
        2,
    );
}
