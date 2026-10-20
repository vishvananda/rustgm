fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
enum E<T> {
    V(T),
}
impl<T: Copy> E<T> {
    fn get(&self) -> T {
        match self {
            E::V(x) => *x,
        }
    }
}
fn f(s: &E<u8>) -> u8 {
    s.get()
}
fn main() {
    check(f(&E::V(7)) == 7);
}
