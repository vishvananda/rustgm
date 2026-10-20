trait A{type Out<'x,T>where T:'x;fn f<'a,'b,T,U>(x:&'a T,y:Self::Out<'b,U>)->Self::Out<'a,T>;}
fn main(){}
