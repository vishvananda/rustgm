trait A{type Out<'a>;fn f(x:for<'a>fn(&'a Self)->Self::Out<'a>);}
fn main(){}
