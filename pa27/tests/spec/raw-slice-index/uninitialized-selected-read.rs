#![allow(dead_code,unused_mut,invalid_reference_casting)]
union U{p:*const [u32],parts:(*const u32,usize)}const X:u32={let a:[u32;2];let u=U{parts:(&raw const a as *const u32,2)};unsafe{(*u.p)[0]}};fn main(){let _=X;}
