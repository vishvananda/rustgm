struct S<T: ?Sized> {
    tail: T,
}
fn f(x: &mut S<[u8; 2]>) -> *mut S<[u8]> {
    x
}
fn main() {}
