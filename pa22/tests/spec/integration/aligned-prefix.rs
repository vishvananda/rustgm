fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
#[repr(C, align(64))]
struct S<T: ?Sized> {
    tag: u8,
    tail: T,
}
fn read(s: &S<[u8]>) {
    let base = s as *const S<[u8]> as *const u8 as usize;
    check((&raw const s.tail[0]) as usize - base == 1);
    check(s.tail[1] == 7);
}
fn main() {
    read(&S {
        tag: 3,
        tail: [5, 7],
    });
}
