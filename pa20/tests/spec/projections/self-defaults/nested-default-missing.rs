trait Source {
    type Item;
}
trait Other<Q = <Self as Source>::Item> {
    type X;
}
trait Fill<R = <Self as Other>::X> {
    type Out;
}
struct Short;
struct Marked;
impl Source for Short {
    type Item = Marked;
}
impl Other<u8> for Short {
    type X = u8;
}
impl Fill for Short {
    type Out = u16;
}
fn main() {}
