#[repr(C)]struct A{a:u32}#[repr(C)]struct X{a:A}extern "C" fn f(x:X)->X{x}fn main(){let _=f(X{a:A{a:7}});}
