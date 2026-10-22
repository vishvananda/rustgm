#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
union U{chr:char,bits:u32}const X:u32={let mut u=U{bits:0};unsafe{core::ptr::write_bytes(&raw mut u.chr,255,1);u.bits}};fn main(){check(X==u32::MAX);}
