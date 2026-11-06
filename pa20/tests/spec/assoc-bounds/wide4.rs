trait A{type Out;}trait B<const N:usize>{}impl<const N:usize>B<N>for u8{}struct S;impl A for S{type Out=u8;}fn f<T:A<Out:B<0>+B<1>+B<2>+B<3>>>(){}fn main(){f::<S>();}
