#![allow(dead_code,unused_mut)]
union U{small:u8,wide:u32}const X:u32={let mut u=U{small:0};unsafe{core::ptr::write_bytes(&raw mut u.wide,1,1);u.wide}};fn main(){let _=X;}
