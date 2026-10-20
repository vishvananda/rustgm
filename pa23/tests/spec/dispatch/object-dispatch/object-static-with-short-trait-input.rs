trait A<'a>{}fn f<'a>(x:&'static(dyn A<'a>+'static))->&'static(dyn A<'a>+'static){x}fn main(){}
