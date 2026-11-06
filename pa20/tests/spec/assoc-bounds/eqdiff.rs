trait A{type Out;}fn f<T:A<Out=u8,Out=u16>>(){}fn main(){}
