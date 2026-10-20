fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct D<'a>(&'a mut u8, u8);
impl<'a> Drop for D<'a> {
    fn drop(&mut self) {
        *self.0 = self.1;
    }
}
struct S<T: ?Sized> {
    tail: T,
}
fn read(s: &S<[D<'_>]>) -> u8 {
    s.tail[0].1
}
fn main() {
    let mut n = 0;
    check(
        read(&S {
            tail: [D(&mut n, 7)],
        }) == 7,
    );
    check(n == 7);
}
