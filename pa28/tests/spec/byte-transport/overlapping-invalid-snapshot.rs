#![allow(dead_code)]
union U{bits:u8,value:bool} const X:u8={let source=U{bits:2};let mut array=[false,true,false,true];let mut target=U{bits:0};unsafe{core::ptr::copy(&raw const source.value,&raw mut array[0],1);core::ptr::copy(&raw const array[0],&raw mut array[1],3);core::ptr::copy(&raw const array[1],&raw mut target.value,1);target.bits}};
fn main(){let _=X;}
