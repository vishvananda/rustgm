trait A<'a>:'a{}trait B:for<'a>A<'a>{}fn prove<T:?Sized+'static>(_:&T){}fn f<'x>(x:&'x(dyn B+'x)){prove(x)}fn main(){}
