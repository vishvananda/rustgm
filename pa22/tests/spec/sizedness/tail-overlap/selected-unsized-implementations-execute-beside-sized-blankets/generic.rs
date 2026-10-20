fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T> {
    prefix: T,
    tail: [u8],
}
trait B {
    fn n() -> u8;
}
impl<T> B for T {
    fn n() -> u8 {
        1
    }
}
impl<T> B for S<T> {
    fn n() -> u8 {
        3
    }
}
fn main() {
    check(<S<u8> as B>::n() == 3);
    check(<S<u16> as B>::n() == 3);
    check(<u8 as B>::n() == 1);
}
