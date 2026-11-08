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
impl Fill<u8> for Other {
    type Out = u8;
    fn fill(_: u8) -> Self {
        Other
    }
}
fn qualified(value: <Other as Fill>::Out) -> u8 {
    value
}
fn main() {}
