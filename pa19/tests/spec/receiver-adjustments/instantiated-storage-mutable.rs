fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T>(T);
impl<T: Copy> S<T> {
    fn set(&mut self, x: T) {
        self.0 = x;
    }
}
fn f(s: &mut S<u8>) {
    s.set(7);
}
fn main() {
    let mut s = S(3);
    f(&mut s);
    check(s.0 == 7);
}
