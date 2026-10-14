type A<'a,T>=&'a T;fn f(x:A<u8>)->&u8{x}
fn main(){}
