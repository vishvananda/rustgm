#![allow(dead_code,unused_mut)]
mod core{pub mod ptr{pub fn write_bytes<T>(_: *mut T,_:u8,_:usize){}}}const X:()={let mut x=0u32;core::ptr::write_bytes(&raw mut x,0,1)};fn main(){let _=X;}
