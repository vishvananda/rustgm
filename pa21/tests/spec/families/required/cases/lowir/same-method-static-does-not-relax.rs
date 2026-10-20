trait A<T>{type Out<'x>;fn f<'a>(x:&'a T,y:Self::Out<'static>)->Self::Out<'a>;}
fn main(){}
