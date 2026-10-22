#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static A:u32=37;struct S{r:&'static u32,n:u32}const X:u32={let mut s=S{r:&A,n:0};unsafe{core::ptr::write_bytes(&raw mut s,1,1);}s.r=&A;s.n+*s.r};fn main(){check(X==0x01010101+37);}
