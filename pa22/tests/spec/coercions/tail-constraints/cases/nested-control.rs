struct S<P, T: ?Sized> {
    prefix: P,
    tail: T,
}
struct W<T: ?Sized> {
    tail: S<u8, T>,
}
fn f(s: &W<[u8; 2]>) -> &W<[u8]> {
    s
}
fn main() {}
