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
    fn get(&self) -> T {
        self.0
    }
}
fn f<T: Copy>(s: &S<T>) -> T {
    s.get()
}
fn main() {
    check(f(&S(5u8)) == 5);
}
