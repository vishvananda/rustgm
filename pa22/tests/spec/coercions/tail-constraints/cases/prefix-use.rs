struct S<T: ?Sized> {
    prefix: *const T,
    tail: T,
}
fn f(s: &S<[u8; 2]>) -> &S<[u8]> {
    s
}
fn main() {}
