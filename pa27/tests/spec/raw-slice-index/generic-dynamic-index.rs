#![allow(dead_code,unused_mut,invalid_reference_casting)]
static A:[u32;3]=[3,5,7];const fn get(p:*const [u32],index:usize)->u32{unsafe{(*p)[index]}}const X:u32={let r:&[u32]=&A;get(r as *const [u32],2)};fn main(){let _=X;}
