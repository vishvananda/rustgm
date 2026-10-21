#![allow(invalid_reference_casting)]
struct S{x:u32}const X:u32={let s=S{x:7};let r=&s;let p=(&raw const r.x) as *mut u32;unsafe{*p=9;}s.x};fn main(){let _=X;}
