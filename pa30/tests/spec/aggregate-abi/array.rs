#[repr(C)]struct X{a:[u32;3]}extern "C" fn f(x:X)->X{x}fn main(){let _=f(X{a:[1,2,3]});}
