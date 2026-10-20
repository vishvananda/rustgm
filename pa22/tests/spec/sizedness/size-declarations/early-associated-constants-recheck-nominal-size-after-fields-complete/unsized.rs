struct Array<const N: usize>([u8; N]);
struct S {
    a: Array<{ <S as A>::V }>,
    tail: str,
}
trait A {
    const V: usize;
}
impl<T> A for T {
    const V: usize = 1;
}
fn main() {}
