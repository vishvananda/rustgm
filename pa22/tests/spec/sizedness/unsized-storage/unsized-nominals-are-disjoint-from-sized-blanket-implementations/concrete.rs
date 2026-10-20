fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S {
    prefix: u16,
    tail: str,
}
trait Flag {
    fn n() -> u8;
}
impl<T> Flag for T {
    fn n() -> u8 {
        1
    }
}
impl Flag for S {
    fn n() -> u8 {
        2
    }
}
fn main() {
    check(<S as Flag>::n() == 2);
    check(<u8 as Flag>::n() == 1);
}
