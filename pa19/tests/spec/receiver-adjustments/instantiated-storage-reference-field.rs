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
fn f<'a>(s: &S<&'a u8>) -> &'a u8 {
    s.get()
}
fn main() {
    let n = 7;
    check(*f(&S(&n)) == 7);
}
