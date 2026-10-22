#![allow(dead_code,unused_mut)]
const X:u32={let mut x=0u32;unsafe{core::ptr::write_bytes(&raw mut x,0,18446744073709551615usize);}x};fn main(){let _=X;}
