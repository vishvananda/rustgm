trait A<'a>:'a{}fn prove<'a,T:?Sized+'a>(_:&T){}fn f<'a:'b,'b>(x:&'b(dyn A<'a>+'b)){prove::<'a,_>(x)}fn main(){}
