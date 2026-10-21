#![allow(invalid_reference_casting)]
union U<T:Copy>{a:T,b:u8}const fn address<T:Copy>(u:&U<T>)->*mut T{(&raw const u.a) as *mut T}const X:u32={let u=U{a:83u32};let p=address(&u);unsafe{*p=9;}0};fn main(){let _=X;}
