trait A<'a>{}fn prove<T:'static>(){}fn f<'a>(_:&'static(dyn A<'a>+'static)){prove::<&'a u8>();}fn main(){}
