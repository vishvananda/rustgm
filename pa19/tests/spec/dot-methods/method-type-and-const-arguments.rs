fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl S{fn f<T:Copy,const N:usize>(&self,x:T)->[T;N]{[x;N]}}fn main(){check(S.f::<u8,2>(7)[1]==7);}