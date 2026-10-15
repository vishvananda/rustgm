fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl S{fn f<T:Copy,const N:usize>(&self,x:&[T;N])->T{x[N-1]}}fn main(){check(S.f::<u16,_>(&[2,517])==517);}