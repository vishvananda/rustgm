#![allow(dead_code,unused_mut)]
const X:()=unsafe{core::ptr::write_bytes(0 as *mut (),255,1)};fn main(){let _=X;}
