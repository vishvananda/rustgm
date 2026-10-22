#![allow(dead_code,unused_mut)]
const X:u32={let mut x=0u32;unsafe{core::ptr::write_bytes(&raw mut x,0,usize::MAX);}x};fn main(){let _=X;}
