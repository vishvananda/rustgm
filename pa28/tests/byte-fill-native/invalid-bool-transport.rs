#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
union U{bit:bool,byte:u8}const X:u8={let mut u=U{byte:0};unsafe{core::ptr::write_bytes(&raw mut u.bit,2,1);u.byte}};fn main(){check(X==2);}
