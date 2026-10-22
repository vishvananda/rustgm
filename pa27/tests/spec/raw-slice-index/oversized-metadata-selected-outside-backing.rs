#![allow(dead_code,unused_mut,invalid_reference_casting)]
static A:[u8;3]=[3,5,7];union U{p:*const [u8],words:[usize;2]}const X:u8={let r:&[u8]=&A;let mut u=U{p:r};unsafe{u.words[1]=5;(*u.p)[4]}};fn main(){let _=X;}
