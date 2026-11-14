#![feature(async_drop)]
#![no_std]
pub use core::future::{Future,async_drop_in_place as cleanup};
pub const LANES:usize=4;
pub const FLAG:bool=true;
pub const LETTER:char='x';
pub const NEG:i8=0xFF_u8 as i8;
