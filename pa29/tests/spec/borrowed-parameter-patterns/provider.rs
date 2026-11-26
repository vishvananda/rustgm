pub fn through_parameter<T>(ref mut value: T, callback: fn(&mut T) -> u8) -> u8 {
    callback(value)
}

pub fn through_field<T>((ref mut value,): (T,), callback: fn(&mut T) -> u8) -> u8 {
    callback(value)
}
