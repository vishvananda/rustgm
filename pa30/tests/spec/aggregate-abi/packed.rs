#[repr(C,packed)]struct X{a:u8,b:u64}extern "C" fn f(x:X)->X{x}fn main(){let _=f(X{a:1,b:2});}
