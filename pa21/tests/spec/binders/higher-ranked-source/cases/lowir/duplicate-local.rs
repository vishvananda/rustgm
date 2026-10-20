trait A<'a>{}fn f<T:for<'a,'a>A<'a>>(){}fn main(){}
