#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:u8={let r:&[u8]=b"abc";let p=r as *const [u8] as *mut [u8];unsafe{(*p)[1]=7;(*p)[1]}};fn main(){let _=X;}
