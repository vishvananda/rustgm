#![allow(dead_code,unused_mut)]
const X:u32={let mut x=1u32;let p=&raw mut x;{let q=&x;let _=*q;}unsafe{core::ptr::write_bytes(p,7,1);}x};fn main(){let _=X;}
