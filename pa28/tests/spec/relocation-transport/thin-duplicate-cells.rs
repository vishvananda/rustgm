#![allow(dead_code,unused_mut,invalid_reference_casting)]
union U{pointer:*const u32,bytes:[u8;8]}static A:u32=37;const X:u32={let source=U{pointer:&A};let other=U{pointer:&A};let mut target=U{bytes:[0;8]};unsafe{core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],4);core::ptr::copy_nonoverlapping(&raw const other.bytes[4],&raw mut target.bytes[4],4);*target.pointer}};
fn main(){let _=X;}
