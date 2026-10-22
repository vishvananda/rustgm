#![allow(dead_code,unused_mut)]
const X:()=unsafe{core::ptr::write_bytes(1 as *mut (),255,usize::MAX)};fn main(){let _=X;}
