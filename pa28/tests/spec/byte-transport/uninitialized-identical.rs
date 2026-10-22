#![allow(dead_code)]
union U{wide:u32,tiny:u8} const X:u8={let mut target=U{tiny:7};unsafe{core::ptr::copy(&raw const target.wide,&raw mut target.wide,1);target.tiny}};
fn main(){let _=X;}
