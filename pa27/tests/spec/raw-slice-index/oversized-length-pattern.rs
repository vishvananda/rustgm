#![allow(dead_code,unused_mut,invalid_reference_casting)]
static A:[u32;3]=[3,5,7];union U{p:*const [u32],words:[usize;2]}const X:u32={let r:&[u32]=&A;let mut u=U{p:r};unsafe{u.words[1]=5;match *u.p{[_,_,_,_,_]=>37,_=>0}}};fn main(){let _=X;}
