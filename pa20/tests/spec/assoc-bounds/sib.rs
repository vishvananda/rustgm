trait A{type Out;type Sibling;}trait B{}struct S;impl A for S{type Out=u8;type Sibling=u16;}impl B for u16{}fn f<T:A<Out:B>>(){}fn main(){f::<S>();}
