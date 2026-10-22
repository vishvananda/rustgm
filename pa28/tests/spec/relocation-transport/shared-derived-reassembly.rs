#![allow(dead_code,unused_mut,invalid_reference_casting)]
union U{pointer:*mut u32,bytes:[u8;8]}const X:u32={let mut value=37;let source=U{pointer:&value as *const u32 as *mut u32};let mut target=U{bytes:[0;8]};unsafe{core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],4);core::ptr::copy_nonoverlapping(&raw const source.bytes[4],&raw mut target.bytes[4],4);*target.pointer=91;}value};
fn main(){let _=X;}
