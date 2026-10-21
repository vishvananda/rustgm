#![allow(invalid_reference_casting)]
const X:u32={let mut x=7u32;let p=&raw mut x;unsafe{let r=&*p;let q=(r as *const u32)as *mut u32;*q=9;}x};fn main(){let _=X;}
