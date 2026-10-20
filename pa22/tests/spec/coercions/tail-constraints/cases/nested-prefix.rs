struct S<P, T: ?Sized> {
    prefix: *const P,
    tail: T,
}
struct W<T: ?Sized> {
    tail: S<T, T>,
}
fn f(s: &W<[u8; 2]>) -> &W<[u8]> {
    s
}
fn main() {}
