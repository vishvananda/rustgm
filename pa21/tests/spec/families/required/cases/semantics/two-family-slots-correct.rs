trait A<T>{type Out<'x,'y>where T:'x;fn f<'a,'b>(x:&'a T)->Self::Out<'a,'b>;}
fn main(){}
