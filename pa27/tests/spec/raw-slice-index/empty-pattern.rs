#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:u32={let a:[u32;0]=[];let r:&[u32]=&a;let p=r as *const [u32];unsafe{match *p{[]=>37,_=>0}}};fn main(){let _=X;}
