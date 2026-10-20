struct S<T: ?Sized> {
    prefix: *const u8,
    tail: T,
}
fn f(s: &S<[u8; 2]>) -> &S<[u8]> {
    s
}
fn main() {}
