trait A{type Out<'x,T>;fn f<'a,'b,T,U>(x:&'a T,y:&'b U,z:Self::Out<'b,U>)->Self::Out<'a,T>;}
fn main(){}
