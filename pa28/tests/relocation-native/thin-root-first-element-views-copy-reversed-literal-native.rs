#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static A:[u32;2]=[37,91];union U{pointer:*const u32,bytes:[u8;8]}const X:*const u32={let source=U{pointer:&A as *const [u32;2] as *const u32};let other=U{pointer:&A[0]};let mut target=U{bytes:[0;8]};unsafe{core::ptr::copy_nonoverlapping(&raw const other.bytes[4],&raw mut target.bytes[4],4);core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],4);target.pointer}};
fn main(){check(unsafe{*X}==37);}
