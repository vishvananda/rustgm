union U{bits:u32,value:char} const X:u32={let source=U{bits:0x110000};let mut target=U{bits:0};unsafe{core::ptr::copy_nonoverlapping(&raw const source.value,&raw mut target.value,1);target.bits}};
fn main(){check(X==0x110000);}
