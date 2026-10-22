#![allow(dead_code)]
union U{wide:u32,tiny:u8}const X:u32={let source=U{tiny:7};let mut value:u32;unsafe{core::ptr::copy(&raw const source.wide,&raw mut value,1);core::ptr::read(&raw const value)}};
fn main(){let _=X;}
