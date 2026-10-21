#[repr(C)]enum X{A,B}extern "C" fn f(x:X)->X{x}fn main(){let _=f(X::A);}
