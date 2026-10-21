#[repr(C)]struct X{a:u32}extern "C" fn f(x:X)->X{x}const N:X=f(X{a:7});fn main(){}
