#![allow(invalid_reference_casting)]
const X:u32={let x=7u32;let p=(&x) as *const u32 as *mut u32;let t=(p,);unsafe{*t.0=9;}x};fn main(){let _=X;}
