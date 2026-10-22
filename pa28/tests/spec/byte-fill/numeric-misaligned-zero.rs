#![allow(dead_code,unused_mut)]
const X:()=unsafe{core::ptr::write_bytes(1 as *mut u32,0,0)};fn main(){let _=X;}
