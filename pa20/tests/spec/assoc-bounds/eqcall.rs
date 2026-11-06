trait A{type Out;}struct S;impl A for S{type Out=u8;}fn f<T:A<Out=u8,Out=u16>>(){}fn main(){f::<S>();}
