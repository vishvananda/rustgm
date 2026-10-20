fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
#[repr(C)]
struct S<T: ?Sized> {
    tag: u16,
    tail: T,
}
#[repr(C)]
struct W<T: ?Sized> {
    tag: u8,
    tail: S<T>,
}
fn read(s: &W<[u32]>) {
    let base = s as *const W<[u32]> as *const u8 as usize;
    check((&raw const s.tail.tail[0]) as usize - base == 8);
    check(s.tail.tail[1] == 7);
    check(s.tag == 3 && s.tail.tag == 5);
}
fn main() {
    read(&W {
        tag: 3,
        tail: S {
            tag: 5,
            tail: [3, 7],
        },
    });
}
