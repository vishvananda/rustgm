#[repr(C)]struct X{a:u32}#[repr(C)]struct Y{a:u32}extern "C" fn f(x:X)->X{x}fn main(){let _=f(Y{a:7});}
