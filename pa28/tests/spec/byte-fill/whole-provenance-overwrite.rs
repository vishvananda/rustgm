#![allow(dead_code,unused_mut)]
union U{pointer:*const u32,bytes:[u8;8]}static A:u32=37;const X:U={let mut u=U{pointer:&A};unsafe{core::ptr::write_bytes(&raw mut u.bytes[0],0,8);}u};fn main(){let _=X;}
