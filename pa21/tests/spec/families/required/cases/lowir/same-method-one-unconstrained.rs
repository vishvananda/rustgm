trait A<T>{type Out<'x>;fn f<'a,'b>(x:&'a T,y:Self::Out<'b>)->Self::Out<'a>;}
fn main(){}
