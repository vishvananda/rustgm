#[repr(C)]struct X;extern "C" fn f(x:X)->X{x}fn main(){let _=f(X);}
