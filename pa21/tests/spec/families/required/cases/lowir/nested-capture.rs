trait A{type Out<'a>;fn f<'a>(x:&'a Self,y:fn()->Self::Out<'a>);}
fn main(){}
