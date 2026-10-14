type A<'a,T=A<'a,&'a u8>>=T;fn f<'a>(x:A<'a>)->&'a u8{x}
fn main(){}
