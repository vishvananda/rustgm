fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T:Copy>(T,T);fn make<T:Copy>(x:T)->S<T>{S(x,x)}fn main(){let a=make(20u8);check(a.0==20&&a.1==20);}
