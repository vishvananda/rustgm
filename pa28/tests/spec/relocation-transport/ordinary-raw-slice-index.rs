#![allow(dead_code,unused_mut)]
static A:[u8;3]=[3,5,7];const X:u8={let a:&[u8]=&A;let pointer=a as *const [u8];unsafe{(*pointer)[1]}};
fn main(){let _=X;}
