fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
#[repr(align(64))]
struct A(u8);
#[repr(C)]
struct S<T: ?Sized> {
    tag: u8,
    tail: T,
}
fn read(s: &S<[A]>) {
    check(s.tag == 3);
    check(s.tail[1].0 == 7);
    let base = s as *const S<[A]> as *const u8 as usize;
    let tail = (&raw const s.tail[0]) as usize;
    check(tail - base == 64);
}
fn main() {
    let s = S {
        tag: 3,
        tail: [A(5), A(7)],
    };
    read(&s);
}
