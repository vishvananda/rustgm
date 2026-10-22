#![allow(dead_code,unused_mut)]
union U{bit:bool,byte:u8}const X:bool={let mut u=U{byte:2};unsafe{core::ptr::write_bytes(&raw mut u.bit,0,1);u.bit}};fn main(){let _=X;}
