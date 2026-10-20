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
fn f(s: &&S<u8>) -> u8 {
    s.get()
}
fn main() {
    let s = S(3);
    check(f(&&s) == 3);
}
