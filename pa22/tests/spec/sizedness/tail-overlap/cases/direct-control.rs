struct S<T: ?Sized> {
    tail: T,
}
trait B {}
impl<T> B for T {}
impl B for S<[u8]> {}
fn main() {}
