trait Q<'p,T>{type Out<'a,U> where T:'p,U:'a;}struct S;impl<'p,T>Q<'p,T>for S{type Out<'a,U>=(&'p T,&'a U) where T:'p,U:'a;}
fn main(){}
