trait A{type Out;}fn f<T:A<Out=u8,Out=u8>>(){}fn main(){}
