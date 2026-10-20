fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T, const N: usize>([T; N]);
impl<T: Copy, const N: usize> S<T, N> {
    fn get(&self) -> T {
        self.0[0]
    }
}
fn f(s: &S<u8, 2>) -> u8 {
    s.get()
}
fn main() {
    check(f(&S([3, 4])) == 3);
}
