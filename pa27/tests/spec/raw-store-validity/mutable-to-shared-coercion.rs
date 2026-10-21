#![allow(invalid_reference_casting)]
const fn addr(x:&u32)->*mut u32{x as *const u32 as *mut u32}const X:u32={let mut x=7;let p=addr(&mut x);unsafe{*p=9;}x};fn main(){let _=X;}
