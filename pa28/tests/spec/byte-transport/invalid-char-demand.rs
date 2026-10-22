#![allow(dead_code)]
union U{bits:u32,value:char} const X:char={let source=U{bits:0x110000};let mut target=U{bits:0};unsafe{core::ptr::copy_nonoverlapping(&raw const source.value,&raw mut target.value,1);target.value}};
fn main(){let _=X;}
