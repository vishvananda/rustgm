type A<T=B<u8>>=T;type B<U=A<u8>>=U;fn f(x:A)->B{x}
fn main(){}
