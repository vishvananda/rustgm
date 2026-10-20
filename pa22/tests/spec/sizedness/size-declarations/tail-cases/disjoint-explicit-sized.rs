struct S {
    tail: str,
}
trait A {}
impl<T: ?Sized> A for T where T: Sized {}
impl A for S {}
fn main() {}
