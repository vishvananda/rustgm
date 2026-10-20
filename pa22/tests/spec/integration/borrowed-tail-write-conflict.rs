struct S<T: ?Sized> {
    tail: T,
}
fn f(s: &mut S<[u8]>) -> u8 {
    let r = &s.tail[0];
    s.tail[0] = 7;
    *r
}
fn main() {}
