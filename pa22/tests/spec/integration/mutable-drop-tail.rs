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
    tag: u8,
    tail: T,
}
fn edit(s: &mut S<[D<'_>]>) {
    s.tail[0].1 = 9;
    s.tag = 5;
}
fn main() {
    let mut n = 0;
    {
        let mut s = S {
            tag: 3,
            tail: [D(&mut n, 7)],
        };
        edit(&mut s);
        check(s.tag == 5);
    }
    check(n == 9);
}
