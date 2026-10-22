union U{bits:u8,value:bool} struct S{a:bool,p:*const u8} const X:u8={let value=42;let source=U{bits:2};let mut target=S{a:false,p:&value};unsafe{core::ptr::copy(&raw const source.value,&raw mut target.a,1);}target.a=true;unsafe{*target.p}};
fn main(){check(X==42);}
