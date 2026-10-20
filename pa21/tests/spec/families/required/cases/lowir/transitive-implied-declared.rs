trait A<T>{type Out<'x>where T:'x;fn f<'a:'b,'b>(x:&'a T)->Self::Out<'b>;}
fn main(){}
