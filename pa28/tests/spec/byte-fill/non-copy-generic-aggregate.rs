#![allow(dead_code,unused_mut)]
struct Pair{a:u32,b:u32}const fn fill<T>(p:*mut T){unsafe{core::ptr::write_bytes(p,3,1);}}const X:Pair={let mut p=Pair{a:0,b:1};fill(&raw mut p);p};fn main(){let _=X;}
