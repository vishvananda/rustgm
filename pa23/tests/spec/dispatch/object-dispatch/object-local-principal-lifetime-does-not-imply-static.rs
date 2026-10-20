trait A<'a>:'a{}fn prove<T:?Sized+'static>(_:&T){}fn f<'x>(x:&'x(dyn for<'a>A<'a>+'x)){prove(x)}fn main(){}
