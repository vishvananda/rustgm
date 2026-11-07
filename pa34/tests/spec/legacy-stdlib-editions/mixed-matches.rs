#![no_std]
pub fn small(x: u8) -> bool { matches!(x, 0 | 1) }
