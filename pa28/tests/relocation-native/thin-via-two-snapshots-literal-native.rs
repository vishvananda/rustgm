#![allow(dead_code,unused_mut,invalid_reference_casting)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
union U{pointer:*const u32,bytes:[u8;8]}static A:u32=37;static B:u32=91;const X:u32={let source=U{pointer:&A};let mut low=U{bytes:[0;8]};let mut high=U{bytes:[0;8]};let mut target=U{bytes:[0;8]};unsafe{core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut low.bytes[0],4);core::ptr::copy_nonoverlapping(&raw const source.bytes[4],&raw mut high.bytes[4],4);core::ptr::copy_nonoverlapping(&raw const low.bytes[0],&raw mut target.bytes[0],4);core::ptr::copy_nonoverlapping(&raw const high.bytes[4],&raw mut target.bytes[4],4);*target.pointer}};
fn main(){check(X==37);}
