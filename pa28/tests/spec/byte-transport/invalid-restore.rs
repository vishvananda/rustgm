#![allow(dead_code)]
union U{bits:u8,value:bool} const X:bool={let source=U{bits:2};let mut target=false;unsafe{core::ptr::copy(&raw const source.value,&raw mut target,1);core::ptr::write(&raw mut target,true);}target};
fn main(){let _=X;}
