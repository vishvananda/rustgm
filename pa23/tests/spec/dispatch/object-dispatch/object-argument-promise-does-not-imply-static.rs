trait A<T>where T:'static{}fn prove<T:?Sized+'static>(_:&T){}fn f<'x>(x:&'x(dyn A<u8>+'x)){prove(x)}fn main(){}
