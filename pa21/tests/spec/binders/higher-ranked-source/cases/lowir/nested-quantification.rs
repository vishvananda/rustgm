trait A<'a,'b>{}fn f<T>()where for<'a>T:for<'b>A<'a,'b>{}fn main(){}
