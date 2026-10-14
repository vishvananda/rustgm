type A<T=A<u8>> where A<T>:Copy = T;fn f(x:A)->u8{x}
fn main(){}
