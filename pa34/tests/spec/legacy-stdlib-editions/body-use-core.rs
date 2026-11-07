#![no_std]
pub fn first() -> u8 {
    use core::option::Option;
    let value: Option<u8> = None;
    match value { Some(v) => v, None => 0 }
}
