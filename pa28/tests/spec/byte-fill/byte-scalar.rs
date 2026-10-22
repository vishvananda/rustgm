#![allow(dead_code,unused_mut)]
const X:u32={let mut x=0u32;unsafe{core::ptr::write_bytes(&raw mut x,0xa7,1);}x};fn main(){let _=X;}
