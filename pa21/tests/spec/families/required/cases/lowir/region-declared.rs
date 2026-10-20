trait A{type Out<'a,'b>where 'a:'b;fn f<'a:'b,'b>(x:Self::Out<'a,'b>);}
fn main(){}
