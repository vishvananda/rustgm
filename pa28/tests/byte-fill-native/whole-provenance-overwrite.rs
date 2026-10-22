#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
union U{pointer:*const u32,bytes:[u8;8]}static A:u32=37;const X:U={let mut u=U{pointer:&A};unsafe{core::ptr::write_bytes(&raw mut u.bytes[0],0,8);}u};fn main(){check(unsafe{X.bytes[0]}==0);check(unsafe{X.bytes[7]}==0);}
