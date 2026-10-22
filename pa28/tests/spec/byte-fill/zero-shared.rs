#![allow(dead_code,unused_mut)]
const X:u32={let x=7u32;let p=&x as *const u32 as *mut u32;unsafe{core::ptr::write_bytes(p,255,0);}x};fn main(){let _=X;}
