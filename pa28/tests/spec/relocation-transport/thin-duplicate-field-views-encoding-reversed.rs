#![allow(dead_code,unused_mut)]
struct S{value:u32}static A:S=S{value:37};union U{pointer:*const u32,bytes:[u8;8]}const X:*const u32={let other=U{pointer:&A.value};let source=U{pointer:&A.value};let mut target=U{bytes:[0;8]};unsafe{core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],4);core::ptr::copy_nonoverlapping(&raw const other.bytes[4],&raw mut target.bytes[4],4);target.pointer}};
fn main(){let _=X;}
