#[repr(packed)]
struct S<T: ?Sized> {
    tail: T,
}
fn main() {}
