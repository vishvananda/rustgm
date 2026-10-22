#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const X:u32={let mut x=0u32;unsafe{core::ptr::write_bytes(&raw mut x,0xa7,1);}x};fn main(){check(X==0xa7a7a7a7);}
