struct S {
    tail: str,
}
trait A {}
impl<T> A for T {}
impl A for S {}
fn main() {}
