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
fn a() -> u8 {
    3
}
fn b() -> u8 {
    5
}
fn read(s: &S<[fn() -> u8]>) -> u8 {
    s.tail[0]() + s.tail[1]()
}
fn main() {
    check(
        read(&S {
            tag: 7,
            tail: [a, b],
        }) == 8,
    );
}
