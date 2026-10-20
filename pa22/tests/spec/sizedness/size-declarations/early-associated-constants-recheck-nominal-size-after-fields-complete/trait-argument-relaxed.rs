struct Array<const N: usize>([u8; N]);
struct S {
    a: Array<{ <() as A<S>>::V }>,
    tail: str,
}
trait A<T: ?Sized> {
    const V: usize;
}
impl<T: ?Sized> A<T> for () {
    const V: usize = 1;
}
fn main() {}
