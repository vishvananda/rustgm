fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T>S<T>{fn make(x:T)->Self{Self(x)}}fn main(){check(S::make(7u8).0==7);check(S::make(517u16).0==517);}
