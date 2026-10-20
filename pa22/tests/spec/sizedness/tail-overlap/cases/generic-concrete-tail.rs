struct S<T> {
    prefix: T,
    tail: [u8],
}
trait B {}
impl<T> B for T {}
impl<T> B for S<T> {}
fn main() {}
