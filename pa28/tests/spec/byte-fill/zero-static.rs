#![allow(dead_code,unused_mut)]
static S:u32=7;const X:()=unsafe{core::ptr::write_bytes(&S as *const u32 as *mut u32,0,0)};fn main(){let _=X;}
