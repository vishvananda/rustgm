fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>{v:T}impl<T>S<T>{const N:u8=7;const M:u8=match 7{Self::N=>1,_=>2};}fn main(){check(S::<u8>::M==1);check(S::<u16>::M==1);}