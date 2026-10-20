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
fn main() {
    let mut n = 0;
    {
        let s: &S<[D<'_>]> = &S {
            tag: 3,
            tail: [D(&mut n, 7)],
        };
        check(s.tag == 3);
        check(s.tail[0].1 == 7);
    }
    check(n == 7);
}
