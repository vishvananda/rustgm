fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const N:u8=7;struct S<T>{v:T}impl<T>S<T>{const M:u8=match 7{N=>1,_=>2};}fn main(){check(S::<u8>::M==1);check(S::<u16>::M==1);}