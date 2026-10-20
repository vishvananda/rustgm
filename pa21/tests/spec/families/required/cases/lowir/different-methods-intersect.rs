trait A<T>{type Out<'a>;fn f<'a>(x:&'a T)->Self::Out<'a>;fn g<'b>(x:Self::Out<'b>);}
fn main(){}
