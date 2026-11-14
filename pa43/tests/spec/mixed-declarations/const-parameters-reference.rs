#![feature(async_drop)]
#![no_std]
pub use core::future::{Future,async_drop_in_place as cleanup};
pub struct Tag<const N:usize>;
impl<const N:usize>Tag<N>{pub fn new()->Self{Tag}}
pub fn three()->Tag<3>{Tag}
