struct S<T: ?Sized> {
    head: u8,
    tail: T,
}
fn f<T: ?Sized>(p: *const T) -> *const S<T> {
    p as *const S<T>
}
fn main() {}
