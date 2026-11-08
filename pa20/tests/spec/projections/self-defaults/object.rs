trait Source {
    type Item;
}
trait Fill<R = <Self as Source>::Item> {
    type Out;
    fn fill(item: R) -> Self
    where
        Self: Sized;
}
fn object(_: &dyn Fill<Out = u8>) {}
fn main() {}
