trait A<'a>{}fn f<T>()where &'a T:for<'a>A<'a>{}fn main(){}
