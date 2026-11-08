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
struct Other;
impl Fill for Other {
    type Out = u8;
    fn fill(_: u8) -> Self {
        Other
    }
}
fn main() {}
