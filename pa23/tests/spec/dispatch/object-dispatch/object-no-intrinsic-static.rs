trait A{}fn prove<T:?Sized+'static>(_:&T){}fn f<'a>(x:&'a(dyn A+'a)){prove(x)}fn main(){}
