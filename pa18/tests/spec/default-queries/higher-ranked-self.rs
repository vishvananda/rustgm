type A<T=A<for<'a>fn(&'a u8)->&'a u8>>=T;fn f(x:A)->for<'a>fn(&'a u8)->&'a u8{x}
fn main(){}
