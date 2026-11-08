trait Super<T: ?Sized> {
    fn sup(&self) -> u8 {
        1
    }
}
trait Sub: Super<Self> {}
impl Super<u8> for u8 {}
impl Sub for u8 {}
fn through<T: Sub>(value: &T) -> u8 {
    <T as Super<T>>::sup(value)
}
fn main() {
    let _ = through(&1u8);
}
