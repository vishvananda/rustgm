#![allow(dead_code)]
union U{wide:u32,tiny:u8} const X:u32={let source=U{tiny:7};let mut target=[1u32,2,3];unsafe{core::ptr::copy(&raw const source.wide,&raw mut target[1],1);}target[1]=7;let [_,ref rest @ ..]=target;rest[0]};
fn main(){let _=X;}
