#![allow(invalid_reference_casting)]
const fn write(p:*mut u32){unsafe{*p=9;}}const X:u32={let x=7u32;let p=(&x) as *const u32 as *mut u32;write(p);x};fn main(){let _=X;}
