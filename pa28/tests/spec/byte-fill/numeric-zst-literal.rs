#![allow(dead_code,unused_mut)]
const X:()=unsafe{core::ptr::write_bytes(1 as *mut (),255,18446744073709551615usize)};fn main(){let _=X;}
