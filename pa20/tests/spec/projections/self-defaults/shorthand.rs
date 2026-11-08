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
impl Fill<Marked> for Short {
    type Out = u8;
    fn fill(_: Marked) -> Self {
        Short
    }
}
fn shorthand<T: Source + Fill>(value: T::Out) -> T::Out {
    value
}
fn main() {
    let _ = shorthand::<Short>(1);
}
