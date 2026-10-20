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
fn pass(p: *mut S<[u8]>) -> *const S<[u8]> {
    p
}
fn main() {
    let mut a = S { tail: [3u8, 5] };
    let r = &mut a;
    let p: *mut S<[u8]> = r;
    r.tail[0] = 9;
    let q: *const S<[u8]> = &a;
    check(pass(p) == q);
    check(a.tail[0] == 9);
}
