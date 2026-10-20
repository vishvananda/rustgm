struct S<T>(T);
impl<T: Copy> S<T> {
    fn set(&mut self, x: T) {
        self.0 = x;
    }
}
fn f(s: &mut S<u8>) {
    s.set(2u16);
}
fn main() {}
