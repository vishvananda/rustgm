struct S<T>(T);
impl<T: Copy> S<T> {
    fn set(&mut self, x: T) {
        self.0 = x;
    }
}
fn f(s: &S<u8>) {
    s.set(2);
}
fn main() {}
