#![allow(dead_code,unused_mut)]
#![allow(invalid_reference_casting)]
static S:u32=7;const X:()=unsafe{core::ptr::write_bytes(&S as *const u32 as *mut u32,0,1)};fn main(){let _=X;}
