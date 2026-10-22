#![allow(dead_code)]
union U{bits:u8,value:bool} struct S{a:bool,b:u32} const X:S={let source=U{bits:2};let mut target=S{a:false,b:40};unsafe{core::ptr::copy(&raw const source.value,&raw mut target.a,1);}target.b+=2;target.a=true;target};
fn main(){let _=X;}
