#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
union U{bit:bool,byte:u8}const X:bool={let mut u=U{byte:2};unsafe{core::ptr::write_bytes(&raw mut u.bit,0,1);u.bit}};fn main(){check(!X);}
