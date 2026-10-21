struct X{a:u32}extern "C" fn f(x:X)->X{x}fn main(){let _=f(X{a:7});}
