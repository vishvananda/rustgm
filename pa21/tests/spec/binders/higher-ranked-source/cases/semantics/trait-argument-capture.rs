trait A<'a,U>{}impl<'a,U>A<'a,U>for u8{}fn f<'x,T:for<'a>A<'a,&'x u8>>(x:&'x u8){}fn main(){let n=1;f::<u8>(&n);}
