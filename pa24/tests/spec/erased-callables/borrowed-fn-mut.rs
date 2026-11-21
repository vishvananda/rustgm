fn apply(callback: &mut dyn FnMut(u8) -> (u8, u8), input: u8) -> (u8, u8) {
    callback(input)
}
fn bridge<T: FnMut(u8) -> (u8, u8)>(callback: &mut T, input: u8) -> (u8, u8) {
    apply(callback, input)
}
fn divide(value: u8) -> u8 { 1 / value }
fn main() {
    let mut sum = 7;
    let mut add = |value| {
        sum += value;
        (sum, value)
    };
    let mut first: &mut dyn FnMut(u8) -> (u8, u8) = &mut add;
    let result = bridge(&mut first, 5);
    if result.0 != 12 || result.1 != 5 { let _ = divide(0); }
}
