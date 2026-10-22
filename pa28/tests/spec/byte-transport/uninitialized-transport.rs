#![allow(dead_code)]
union U{wide:u32,tiny:u8} const X:u8={let source=U{tiny:7};let mut target=U{wide:0xffffffff};unsafe{core::ptr::copy(&raw const source.wide,&raw mut target.wide,1);target.tiny}};
fn main(){let _=X;}
