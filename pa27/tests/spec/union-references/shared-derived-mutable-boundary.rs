#![allow(invalid_reference_casting)]
union U{a:u32,b:u32}const X:u32={let u=U{a:7};let s=&u;let p=(&raw const s.a)as *mut u32;unsafe{let r=&mut *p;*r=37;u.b}};fn main(){let _=X;}
