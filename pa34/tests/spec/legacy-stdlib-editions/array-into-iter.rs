#![no_std]
pub fn sum() -> u8 {
    let values = [1u8, 2];
    let mut total = 0;
    for value in values.into_iter() { total += *value; }
    total
}
