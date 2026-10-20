trait Long<'a>:'a{}trait A<T>{type Out<'x>;fn f<'a>()->Self::Out<'a>where T:Long<'a>;}
fn main(){}
