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
impl Other<Marked> for Short {
    type X = u8;
}
impl Fill for Short {
    type Out = u16;
}
fn qualified(value: <Short as Fill>::Out) -> <Short as Fill<u8>>::Out {
    value
}
fn main() {
    let _ = qualified(1);
}
