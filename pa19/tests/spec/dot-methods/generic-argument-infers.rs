fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T>S<T>{fn f(&self,x:T)->T{x}}fn main(){let s=S(1);check(s.f(7u8)==7);}