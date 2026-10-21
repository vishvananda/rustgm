#![allow(invalid_reference_casting)]
const X:u32={let x=7u32;let p=(&raw const x) as *mut u32;unsafe{*p=9;}x};fn main(){let _=X;}
