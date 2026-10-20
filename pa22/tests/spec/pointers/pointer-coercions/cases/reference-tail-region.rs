struct S<T: ?Sized> {
    tail: T,
}
fn f<'a>(x: &S<[&'a u8; 2]>) -> *const S<[&'static u8]> {
    x
}
fn main() {}
