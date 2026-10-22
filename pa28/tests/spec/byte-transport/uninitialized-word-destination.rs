#![allow(dead_code)]
union U{wide:u32,tiny:u8}const X:u8={let source=U{tiny:7};let mut value:u32;let mut target=U{wide:0};unsafe{core::ptr::copy(&raw const source.wide,&raw mut value,1);core::ptr::copy(&raw const value,&raw mut target.wide,1);target.tiny}};
fn main(){let _=X;}
