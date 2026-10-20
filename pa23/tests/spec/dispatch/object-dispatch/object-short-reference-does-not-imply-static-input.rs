trait A<'a>{}fn prove<T:'static>(){}fn f<'a,'b>(_:&'b(dyn A<'a>+'static)){prove::<&'a u8>();}fn main(){}
