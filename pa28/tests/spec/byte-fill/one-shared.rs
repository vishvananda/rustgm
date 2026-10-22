#![allow(dead_code,unused_mut)]
#![allow(invalid_reference_casting)]
const X:u32={let x=7u32;let p=&x as *const u32 as *mut u32;unsafe{core::ptr::write_bytes(p,0,1);}x};fn main(){let _=X;}
