#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
union U{small:u8,wide:u32}const X:u32={let mut u=U{small:0};unsafe{core::ptr::write_bytes(&raw mut u.wide,1,1);u.wide}};fn main(){check(X==0x01010101);}
