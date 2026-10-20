trait A<'a>{}fn f<'a,T:for<'a>A<'a>>(x:&'a u8){}fn main(){}
