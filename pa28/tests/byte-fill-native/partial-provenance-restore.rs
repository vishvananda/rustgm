#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
union U{pointer:*const u32,bytes:[u8;8]}static A:u32=37;const X:u32={let source=U{pointer:&A};let mut target=U{pointer:&A};unsafe{core::ptr::write_bytes(&raw mut target.bytes[0],0,4);core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],4);*target.pointer}};fn main(){check(X==37);}
