#![allow(dead_code)]
union U{bits:u8,value:bool} const X:bool={let source=U{bits:2};let mut target=U{bits:0};unsafe{core::ptr::copy(&raw const source.value,&raw mut target.value,1);target.value}};
fn main(){let _=X;}
