#![allow(invalid_reference_casting)]
const X:u32={let mut x=7u32;let r=&x;let p=r as *const u32 as *mut u32;unsafe{*p=9;}x};fn main(){let _=X;}
