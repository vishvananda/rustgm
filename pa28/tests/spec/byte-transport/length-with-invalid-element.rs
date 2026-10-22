#![allow(dead_code)]
union U{bits:u8,value:bool} const X:usize={let source=U{bits:2};let mut target=[false;4];unsafe{core::ptr::copy(&raw const source.value,&raw mut target[0],1);}target.len()};
fn main(){let _=X;}
