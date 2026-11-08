trait Fill<R> {
    fn fill(&self) -> u8 {
        1
    }
}
struct Wrap<T>(T);
impl<T> Fill<T> for Wrap<T> {}
fn need<T: Fill<u8>>(value: &T) -> u8 {
    <T as Fill<u8>>::fill(value)
}
fn main() {
    let _ = need(&Wrap(1u8));
}
