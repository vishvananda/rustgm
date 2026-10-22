#![allow(dead_code,unused_mut)]
union U{bit:bool,byte:u8}const X:u8={let mut u=U{byte:0};unsafe{core::ptr::write_bytes(&raw mut u.bit,2,1);u.byte}};fn main(){let _=X;}
