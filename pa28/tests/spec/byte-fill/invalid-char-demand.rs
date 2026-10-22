#![allow(dead_code,unused_mut)]
union U{chr:char,bits:u32}const X:char={let mut u=U{bits:0};unsafe{core::ptr::write_bytes(&raw mut u.chr,255,1);u.chr}};fn main(){let _=X;}
