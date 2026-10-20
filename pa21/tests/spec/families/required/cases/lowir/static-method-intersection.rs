trait A{type Out<'a>;fn f<'a>(&'a self)->Self::Out<'a>;fn g()->Self::Out<'static>;}
fn main(){}
