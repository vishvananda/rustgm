trait Q<T>{type Out<'a> where T:'a;}struct S;impl<T>Q<T>for S{type Out<'a>=&'a T where T:'a;}
fn main(){}
