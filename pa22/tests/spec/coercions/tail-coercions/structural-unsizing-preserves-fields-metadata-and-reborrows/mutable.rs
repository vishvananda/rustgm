fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
struct S<T: ?Sized> {
    tag: u16,
    tail: T,
}
fn edit(s: &mut S<[u8]>) {
    s.tail[1] = 17;
    s.tag = 9;
}
fn main() {
    let mut s = S {
        tag: 7,
        tail: [1, 3, 9],
    };
    let a = &mut s;
    edit(a);
    check(a.tail[1] == 17);
    check(s.tag == 9);
}
