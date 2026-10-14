fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<T=B<u16>>=T;type B<U=A<u16>>=U;fn f(x:A)->B{x}fn main(){check(f(500)==500);}
