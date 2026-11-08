trait Source {
    type Item;
}
trait Fill<R = <Self as Source>::Item> {
    type Out;
    fn fill(item: R) -> Self;
}
struct Short;
struct Marked;
impl Source for Short {
    type Item = Marked;
}
struct Wrap<T>(T);
impl<T: Source> Source for Wrap<T> {
    type Item = T::Item;
}
impl<T> Fill for Wrap<T> {
    type Out = u8;
    fn fill(_: u8) -> Self {
        loop {}
    }
}
fn main() {}
