#![no_std]
use core::convert::{TryFrom, TryInto};
pub fn bound<T: TryFrom<u8> + TryInto<u8>>() {}
