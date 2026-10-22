#![allow(dead_code,unused_mut)]
#![allow(invalid_null_arguments)]
const X:()=unsafe{core::ptr::write_bytes(0 as *mut u32,0,0)};fn main(){let _=X;}
