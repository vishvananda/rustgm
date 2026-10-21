#![allow(invalid_reference_casting)]
static S:u32=7;const X:u32={let p=(&raw const S) as *mut u32;unsafe{*p=9;}0};fn main(){let _=X;}
