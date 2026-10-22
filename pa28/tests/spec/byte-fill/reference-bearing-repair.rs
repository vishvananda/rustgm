#![allow(dead_code,unused_mut)]
static A:u32=37;struct S{r:&'static u32,n:u32}const X:u32={let mut s=S{r:&A,n:0};unsafe{core::ptr::write_bytes(&raw mut s,1,1);}s.r=&A;s.n+*s.r};fn main(){let _=X;}
