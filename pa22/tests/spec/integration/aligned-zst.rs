fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
#[repr(align(64))]
struct Z;
#[repr(C)]
struct S<T: ?Sized> {
    tag: u8,
    tail: T,
}
fn read(s: &S<[Z]>) {
    check(s.tag == 3);
    match &s.tail {
        [_, _, _] => (),
        _ => check(false),
    }
    let base = s as *const S<[Z]> as *const u8 as usize;
    let tail = (&raw const s.tail[0]) as usize;
    check(tail - base == 64);
    check((&raw const s.tail[0]) == (&raw const s.tail[2]));
}
fn main() {
    read(&S {
        tag: 3,
        tail: [Z, Z, Z],
    });
}
