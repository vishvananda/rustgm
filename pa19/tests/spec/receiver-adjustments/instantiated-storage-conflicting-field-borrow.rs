struct S<T>(T);
impl<T: Copy> S<T> {
    fn set(&mut self, x: T) {
        self.0 = x;
    }
}
fn f(s: &mut S<u8>) -> u8 {
    let r = &s.0;
    s.set(2);
    *r
}
fn main() {}
