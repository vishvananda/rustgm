trait A{type Out;}impl A for u8{type Out=u8;}fn f()where u8:A<Out=for<'a>fn(&'a u8)>{}fn main(){}
