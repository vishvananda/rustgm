#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Pair{a:u32,b:u32}const fn fill<T>(p:*mut T){unsafe{core::ptr::write_bytes(p,3,1);}}const X:Pair={let mut p=Pair{a:0,b:1};fill(&raw mut p);p};fn main(){check(X.a==0x03030303);check(X.b==0x03030303);}
