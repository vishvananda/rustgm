#![allow(dead_code,unused_mut,invalid_reference_casting)]
union U{pointer:*const u32,bytes:[u8;8]}static A:u32=37;static B:u32=91;const X:U={let mut target=U{pointer:&A};unsafe{target.bytes[0]=0;}target};
fn main(){let _=X;}
