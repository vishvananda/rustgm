type A<'a,T=&'a u8>=T;fn f<'b>(x:A<'b>)->&'b u8{x}
fn main(){}
