#[repr(C)]struct X{a:u32}unsafe extern "C"{fn f(x:X)->X;}fn main(){let _=f(X{a:7});}
