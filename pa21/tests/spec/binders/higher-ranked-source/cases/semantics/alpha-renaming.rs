trait A<'a>{}fn f<T>()where for<'a>T:A<'a>,for<'renamed>T:A<'renamed>{}fn main(){}
