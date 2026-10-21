#![allow(invalid_reference_casting)]
const X:u32={let mut x=7u32;let r=&mut x;let s=&r;let p=(&raw const **s)as *mut u32;unsafe{*p=9;}x};fn main(){let _=X;}