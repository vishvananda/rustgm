trait A<'a,'b>:'a+'b{}fn prove<'a,T:?Sized+'a>(_:&T){}fn f<'a,'b,'c>(x:&'c(dyn A<'a,'b>+'c)){prove::<'a,_>(x);prove::<'b,_>(x)}fn main(){}
