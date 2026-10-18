fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T:Copy>S<T>{const C:u8=7;}struct N;fn f(x:u8)->bool{match x{S::<N>::C=>true,_=>false}}fn main(){}