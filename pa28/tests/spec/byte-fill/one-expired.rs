#![allow(dead_code,unused_mut)]
const X:u32={let p={let mut x=0u32;&raw mut x};unsafe{core::ptr::write_bytes(p,0,1);}7};fn main(){let _=X;}
