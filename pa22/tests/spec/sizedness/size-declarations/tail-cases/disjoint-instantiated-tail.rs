struct S<T: ?Sized> {
    tail: T,
}
trait A {}
impl<T> A for T {}
impl A for S<[u8]> {}
fn main() {}
