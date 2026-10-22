#![allow(dead_code,unused_mut,invalid_reference_casting)]
union U{pointer:*const u32,bits:u64}static A:u32=37;const X:u64=unsafe{U{pointer:&A}.bits};
fn main(){let _=X;}
