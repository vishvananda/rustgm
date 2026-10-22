union U{bits:u8,value:bool}const X:u8={let source=U{bits:2};let mut value:bool;let mut target=U{bits:0};unsafe{core::ptr::copy(&raw const source.value,&raw mut value,1);core::ptr::copy(&raw const value,&raw mut target.value,1);target.bits}};
fn main(){check(X==2);}
