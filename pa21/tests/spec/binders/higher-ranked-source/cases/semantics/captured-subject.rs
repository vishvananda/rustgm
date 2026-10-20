trait A<'a>{}fn f<'x,T>(x:&'x T)where &'x T:for<'a>A<'a>{}fn main(){}
