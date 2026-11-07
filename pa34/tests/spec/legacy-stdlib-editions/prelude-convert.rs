#![no_std]
pub fn bound<T: TryFrom<u8> + TryInto<u8>>() {}
