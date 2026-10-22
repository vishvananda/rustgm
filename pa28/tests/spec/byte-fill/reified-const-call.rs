#![allow(dead_code,unused_mut)]
const F:unsafe fn(*mut u32,u8,usize)=core::ptr::write_bytes::<u32>;const X:u32={let mut x=0u32;unsafe{F(&raw mut x,3,1);}x};fn main(){let _=X;}
