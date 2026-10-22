#![allow(dead_code)]
union U{bits:u8,value:bool} const X:bool={let source=U{bits:2};let mut target=[false;131072];unsafe{core::ptr::copy(&raw const source.value,&raw mut target[0],1);}target[0]=true;target[0]};
fn main(){let _=X;}
