trait A{type Out<'a>where Self:'a;fn f<'a>(&'a self)->Self::Out<'a>;}
fn main(){}
