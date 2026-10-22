#![allow(dead_code,unused_mut,invalid_reference_casting)]
union U{pointer:fn()->u32,bytes:[u8;8]}fn answer()->u32{37}const X:U={let source=U{pointer:answer};let mut target=U{bytes:[0;8]};unsafe{core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],4);core::ptr::copy_nonoverlapping(&raw const source.bytes[4],&raw mut target.bytes[4],4);}target};
fn main(){let _=X;}
