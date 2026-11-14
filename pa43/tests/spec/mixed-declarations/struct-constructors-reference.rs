#![feature(async_drop)]
#![no_std]
pub use core::future::{Future,async_drop_in_place as cleanup};
pub struct Wrap(pub u8);
pub fn wrap(x:u8)->Wrap{Wrap(x)}
pub fn unwrap(w:Wrap)->u8{w.0}
