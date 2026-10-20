struct S {
    tail: str,
}
trait A {}
impl<T: ?Sized> A for T {}
impl A for S {}
fn main() {}
