trait A{type Out<'a>;}trait B{fn f<'a,T:A>(x:&'a T)->T::Out<'a>;}
fn main(){}
