#![allow(dead_code,unused_mut)]
#[repr(packed)]struct P{tag:u8,value:u32}const X:()={let mut p=P{tag:0,value:7};unsafe{core::ptr::write_bytes(&raw mut p.value,0,0)}};fn main(){let _=X;}
