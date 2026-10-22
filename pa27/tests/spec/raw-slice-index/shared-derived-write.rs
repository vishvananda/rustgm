#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:u32={let a=[3u32,5,7];let r:&[u32]=&a;let p=r as *const [u32] as *mut [u32];unsafe{(*p)[1]=41;(*p)[1]}};fn main(){let _=X;}
