#![allow(dead_code,unused_mut,invalid_reference_casting)]
static A:[u32;3]=[3,5,7];union U{reference:&'static [u32],words:[usize;2]}const X:u32={let mut u=U{reference:&A};unsafe{u.words[1]=5;u.reference[1]}};fn main(){let _=X;}
