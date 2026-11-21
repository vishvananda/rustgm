fn apply(callback: &dyn Fn() -> u8) -> u8 {
    callback()
}
fn bridge<T: Fn() -> u8>(callback: &T) -> u8 {
    apply(callback)
}
fn divide(value: u8) -> u8 { 1 / value }
fn main() {
    let value = 7;
    let first: &dyn Fn() -> u8 = &|| value;
    let result = bridge(&first);
    if result != 7 { let _ = divide(0); }
}
