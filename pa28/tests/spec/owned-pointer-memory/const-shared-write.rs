#![allow(invalid_reference_casting)]
struct P{a:u32}const X:u32={let p=P{a:7};let raw=&p as *const P as *mut P;unsafe{core::ptr::write(raw,P{a:37});p.a}};fn main(){let _=X;}
