#[repr(C)]union X{a:u64,b:f64}extern "C" fn f(x:X)->X{x}fn main(){let _=f(X{a:7});}
