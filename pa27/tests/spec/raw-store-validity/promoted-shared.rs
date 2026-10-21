#![allow(invalid_reference_casting)]
const R:&u32=&7;const X:u32={let p=R as *const u32 as *mut u32;unsafe{*p=9;}*R};fn main(){let _=X;}
