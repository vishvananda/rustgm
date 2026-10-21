#![allow(invalid_reference_casting)]
union U{a:u32,b:u8}const X:u32={let u=U{a:7};let r=&u;let p=(&raw const r.a) as *mut u32;unsafe{*p=9;}0};fn main(){let _=X;}
