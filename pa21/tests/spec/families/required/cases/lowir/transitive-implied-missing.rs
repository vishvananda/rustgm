trait A<T>{type Out<'x>;fn f<'a:'b,'b>(x:&'a T)->Self::Out<'b>;}
fn main(){}
