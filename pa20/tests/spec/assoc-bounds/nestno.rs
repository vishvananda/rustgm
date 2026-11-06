trait A{type Out;}trait B{type Part;}trait C{}struct S;impl A for S{type Out=u8;}impl B for u8{type Part=u16;}fn f<T:A<Out:B<Part:C>>>(){}fn main(){f::<S>();}
