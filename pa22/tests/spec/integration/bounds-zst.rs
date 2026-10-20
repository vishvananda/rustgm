fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T: ?Sized> {
    tail: T,
}
fn read(s: &S<[()]>, i: usize) {
    let _x = s.tail[i];
}
fn main() {
    read(&S { tail: [(); 3] }, 3);
}
