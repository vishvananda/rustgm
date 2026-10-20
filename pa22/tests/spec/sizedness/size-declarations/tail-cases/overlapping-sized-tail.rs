struct S {
    tail: u8,
}
trait A {}
impl<T> A for T {}
impl A for S {}
fn main() {}
