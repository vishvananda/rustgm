#![allow(dead_code,unused_mut)]
const X:u32={let mut x=[0u32;2];unsafe{core::ptr::write_bytes(&raw mut x[1],0,2);}x[0]};fn main(){let _=X;}
