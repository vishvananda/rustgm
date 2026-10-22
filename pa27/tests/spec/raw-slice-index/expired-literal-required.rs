#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:u32={let p={let a=[3u32,5];let r:&[u32]=&a;r as *const [u32]};unsafe{match *p{[_,5]=>37,_=>0}}};fn main(){let _=X;}
