#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const X:u32={let mut x=1u32;let p=&raw mut x;{let q=&x;let _=*q;}unsafe{core::ptr::write_bytes(p,7,1);}x};fn main(){check(X==0x07070707);}
