fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T: ?Sized> {
    prefix: u16,
    tail: T,
}
trait Flag {
    fn n() -> u8;
}
impl<T> Flag for T {
    fn n() -> u8 {
        1
    }
}
impl Flag for S<[u8]> {
    fn n() -> u8 {
        3
    }
}
fn main() {
    check(<S<[u8]> as Flag>::n() == 3);
    check(<S<[u8; 1]> as Flag>::n() == 1);
}
