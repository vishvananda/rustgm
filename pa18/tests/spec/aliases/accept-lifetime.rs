type A<'a,T>=&'a T;fn f<'b>(x:A<'b,u8>)->&'b u8{x}
fn main(){}
